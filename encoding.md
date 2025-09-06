# Serde‑Columnar Binary Encoding (Self‑Contained)

This document specifies the exact byte format produced by the `serde_columnar`
crate. It is self‑contained: you can implement a compatible encoder/decoder in
any language without depending on Rust or Postcard.

The format is compact and schema‑directed. A top‑level value (a “table”) is a
sequence of fields. Certain fields may contain “containers” whose elements are
stored column‑wise using dedicated codecs.


## Scope and Stability

- This spec describes the current behavior of the `0.3.x` series of
  `serde_columnar` as implemented in this repository.
- The crate is in progress and not yet stable for production use; details may
  change. The compatibility model (optional fields with stable integer indexes)
  is intended to remain.

## Terminology and Conventions

- “Octet” means an 8‑bit byte.
- All multi‑byte integer encodings within this spec are little‑endian
  base‑128 varints (LEB128) unless explicitly stated otherwise.
- “Sequence” means a varint length `L` followed by exactly `L` items, each
  encoded per its type.
- “Byte string” means a varint length `N` followed by exactly `N` octets.
- “Pair (A, B)” means the bytes of `A` immediately followed by the bytes of
  `B` (no extra tag).
- “Table” is the top‑level struct annotated with `#[columnar(ser, de)]`.
- “Row” is a struct annotated with `#[columnar(vec)]` and/or
  `#[columnar(map)]`. Rows only appear inside containers.


## Primitive Encodings

- Unsigned integer `uN`: LEB128 varint, 7 payload bits per octet. The MSB of
  each octet is the continuation bit: 1 means more octets follow; 0 means last.
  Examples: `0 → 00`, `1 → 01`, `127 → 7F`, `128 → 80 01`.
- Signed integer `iN`: ZigZag, then LEB128. ZigZag maps `…,-2,-1,0,1,2,…` to
  `…,3,1,0,2,4,…` via `u = (x << 1) ^ (x >> (N-1))`.
  Examples: `0 → 00`, `-1 → 01`, `1 → 02`, `-2 → 03`, `2 → 04` (all as 1‑octet varints).
- Boolean: single octet `00` (false) or `01` (true).
- Byte string (`bytes`/`Vec<u8>`): `len: varint` followed by `len` octets.
- UTF‑8 string: `len: varint` followed by `len` UTF‑8 octets.
- Sequence (`Vec<T>`): `len: varint` then `len` elements, each encoded as `T`.
- Option<T>: one varint tag, then optional payload. `0 = None`, `1 = Some`,
  and when tag is `1`, the bytes of `T` follow immediately.

Notes:

- When an encoded integer does not fit the consumer’s target type, the decoder
  must raise an error.
- This format does not embed type or schema tags; decoders must know the
  expected types from the schema the data was produced with.


## Containers and Layout

Two container kinds are supported: list‑like and map‑like.

### Vec‑like containers (`class = "vec"`)

Given a struct `Row` annotated with `#[columnar(vec)]` and a field `data:
Vec<Row>` in a table struct annotated with `#[columnar(ser, de)]`, the value of
`data` is a sequence organized as follows:

data := SEQ(
  COL_0, COL_1, ..., COL_{F-1},      // F non‑optional fields of Row
  (opt_index, opt_COL_bytes)*         // 0+ optional fields by mapping
)

- `F` is the number of non‑optional, non‑skipped fields in `Row`.
- Each `COL_i` is a byte string: the codec output for that column, encoded as a
  byte string (length varint + bytes). See “Columns and Codecs”.
- Optional fields of `Row` are not stored positionally. For each present
  optional field, append a pair `(index, bytes)`:
  - `index`: unsigned varint (the stable field index from
    `#[columnar(optional, index = N)]`).
  - `bytes`: byte string containing the codec output for that optional column.

Decoding:

1. Read the `F` non‑optional column byte strings in order and decode each.
2. Read zero or more `(index, bytes)` pairs until the sequence ends, and decode
   any optional columns present.
3. For every optional field absent from the mapping, synthesize a column of
   length `L` (the maximum length among decoded columns) filled with the type’s
   default value.
4. Reconstruct rows by zipping columns element‑wise.


### Map‑like containers (`class = "map"`)

Given a struct `Row` annotated with `#[columnar(map)]` and a field
`data: Map<K, Row>` in a table struct, the value of `data` is:

data := SEQ(
  KEYS,                                // Vec<K> as a sequence
  COL_0, COL_1, ..., COL_{F-1},        // F non‑optional fields of Row
  (opt_index, opt_COL_bytes)*           // 0+ optional fields by mapping
)

- `KEYS` is a `Vec<K>` encoded as a sequence (length varint + each `K`).
- Column handling matches the vec case. `len(KEYS)` determines the expected
  number of elements per column.

Reconstruction zips `KEYS` with the reconstructed rows to build the map.


### Tables (top‑level structs)

A table struct annotated with `#[columnar(ser, de)]` is a sequence of its fields
in declaration order. For each field:

- If `class = "vec"` or `class = "map"`: encode using the container layouts
  above as a nested sequence value.
- Otherwise: encode the value directly using the primitive rules in this spec.

Optional table fields are not stored positionally. After all non‑optional
fields, append `(index, bytes)` pairs where `index` is the stable field index
and `bytes` is a byte string containing the field encoded per this spec.
Missing optionals decode to the type’s default value.


## Columns and Codecs

Each field of a row becomes an independent column. The schema selects the codec
with `#[columnar(strategy = "...")]`. If unspecified, the “Generic” codec is
used. On wire, the result of a codec is carried as a byte string (length varint
then raw bytes). No strategy tag is stored; the schema determines how to decode.


### Generic (no strategy)

Encodes the raw vector of field values as a sequence: `Vec<T>` → `len: varint`
then `len` elements of `T`, each encoded using the primitive rules in this
spec. This is the fallback for complex types and nested containers.


### RLE (`strategy = "Rle"`)

General run‑length encoding for any element type `T`.

Column bytes are a concatenation of runs — there is no outer length header. Each
run starts with a ZigZag+varint `count: isize`:

- Repeated run: `count > 0`. Next are the bytes of a single `value: T`.
- Literal run: `count < 0`. Next are the bytes of exactly `-count` values of
  type `T`, back‑to‑back.

`count = 0` is invalid. Decoding repeats the single value for repeated runs and
copies the literal sequence for literal runs. A safety limit of
`MAX_RLE_COUNT = 1_000_000_000` applies.


### Delta‑RLE (`strategy = "DeltaRle"`)

Optimized for monotonic or slowly changing integers.

Encoding maintains an `absolute` accumulator (starting at `0`). For each value
`v`, compute `delta = v - absolute`, update `absolute = v`, and append `delta`
to an RLE stream of `i128` using the RLE format above. The column bytes are the
underlying RLE bytes.

Decoding reconstructs `absolute += delta` for each delta read from the RLE
decoder and converts back to the target integer type, erroring on overflow.


### Bool‑RLE (`strategy = "BoolRle"`)

Specialized for booleans. The column bytes are a concatenation of unsigned
varint counts — there is no outer length header. Decoding:

1. Initialize `last = true`, `count = 0`.
2. Repeatedly read a varint `n`:
   - Toggle `last = !last`.
   - Emit `n` copies of `last`.
3. Stop at end of input. A safety limit guards against pathological inputs.

The encoder emits a leading `0` count when the first run is `true`. Example:

- Values: `true, true, false, false, false`
- Counts: `[0, 2, 3]`
- Column bytes (hex): `00 02 03`


### Delta‑of‑Delta (`strategy = "DeltaOfDelta"`)

Compact bit‑packed encoding for timestamp‑like `i64` series with approximately
constant step. The first element is stored verbatim; subsequent elements encode
the second difference `Δ² = (v_i - v_{i-1}) - (v_{i-1} - v_{i-2})`.

Column bytes:

- Head: `Option<i64>` (see “Primitive Encodings”). `Some(first)` when non‑empty,
  `None` when empty.
- Trailer: one octet `U` giving the number of valid bits in the final data
  octet, where `U ∈ {0,1,…,8}`. `U = 0` means the bitstream is empty; `U = 8`
  means the last data octet is fully used.
- Bitstream: big‑endian bit‑packed Δ² codes, appended MSB‑first across octets.

Δ² code classes (prefix then payload):

- `0`                              → Δ² = 0 (1 bit total)
- `10`  + 7 bits unsigned         → Δ² ∈ [−63, 64]      (store `Δ² + 63`)
- `110` + 9 bits unsigned         → Δ² ∈ [−255, 256]    (store `Δ² + 255`)
- `1110` + 12 bits unsigned       → Δ² ∈ [−2047, 2048]  (store `Δ² + 2047`)
- `11110` + 21 bits unsigned      → Δ² ∈ [−(2²⁰−1), 2²⁰] (store `Δ² + (2²⁰−1)`)
- `11111` + 64 bits two’s‑complement → Δ² as signed 64‑bit

Decoding:

1. Read head `Option<i64>`. If `None`, the column is empty. If `Some(x)`, set
   `prev = x` and `prev_delta = 0`, and yield `x` as the first value.
2. While bits remain: read a class per the prefixes above. If class `0`, set
   `prev += prev_delta`. Otherwise, read the payload, unbias to get `Δ²`, then
   `prev_delta += Δ²` and `prev += prev_delta`. Yield `prev` each time.


## Optional Fields and Compatibility

To evolve schemas without a separate version number, fields may be marked:

```
#[columnar(optional, index = N)]
```

Rules:

- All non‑optional fields must precede all optional fields in the struct.
- `index` must be unique per struct and stable across versions.

On wire, optional fields are omitted from the positional portion and instead
encoded as `(index, bytes)` pairs after all non‑optional elements. Decoders that
do not know a field’s `index` ignore it. Decoders that expect a field that was
not sent use `Default::default()` for that field.

This makes adding, removing, or reordering optional fields forward‑ and
backward‑compatible, as long as each field’s binary representation remains
compatible (e.g., `u32` → `u64` is fine, but changing the meaning of a codec is
not).


## Borrowing and Zero‑Copy

Fields annotated with `#[columnar(borrow)]` are deserialized by borrowing from
the input buffer where possible (e.g., `Cow<'de, str>` and `Cow<'de, [u8]>`).
This does not change the on‑wire format; it only affects how bytes are mapped to
Rust values during deserialization.


## Iterable Decoding

If a row type is annotated with `#[columnar(iterable)]`, an iterator view is
generated. Calling `serde_columnar::iter_from_bytes::<Table>(&bytes)` returns a
table in which any `class = "vec"` fields annotated with `iter = "Row"` produce
row iterators rather than eagerly allocating `Vec<Row>`.

Iterators consume the same column bytes described above, using streaming
decoders:

- `AnyRleIter<T>` for `Rle`
- `DeltaRleIter<T>` for `DeltaRle`
- `BoolRleIter` for `BoolRle`
- `DeltaOfDeltaIter<T>` for `DeltaOfDelta`
- `GenericIter<T>` for unstrategized columns

The on‑wire layout is identical to the non‑iterable case.


## Error Handling and Limits

- Column decoders validate run counts (`MAX_RLE_COUNT = 1_000_000_000`) to avoid
  malicious memory usage.
- Integer conversions error if a reconstructed value cannot be represented in
  the target type.
- `DeltaOfDelta` rejects inputs with insufficient header/trailer bytes.


## Worked Examples

These examples use the primitive rules above, so they can be reproduced without
Rust or Postcard.

- Bool‑RLE column for values `true, true, false, false, false`:
  - Counts: `[0, 2, 3]`
  - Column bytes: `00 02 03`

- Encoding a simple vec‑like container with a single boolean column using
  Bool‑RLE. Let `rows: Vec<Row>` where `Row { #[columnar(strategy = "BoolRle")] b: bool }`
  and values are the five booleans above. The container is a sequence with one
  element (one column). That element is a byte string carrying the Bool‑RLE
  bytes. Therefore the container bytes are:
  - Sequence length: `01`
  - Column byte string length: `03`
  - Column payload: `00 02 03`
  - Full bytes (hex): `01 03 00 02 03`

Implementations can use this as a cross‑check.


## Worked Example (Informal)

Consider these types:

```rust
#[columnar(vec, ser, de)]
struct Row {
  #[columnar(strategy = "Rle")]        name: String,
  #[columnar(strategy = "DeltaRle")]   id:   u64,
  #[columnar(optional, index = 0)]      note: String,
}

#[columnar(ser, de)]
struct Table {
  #[columnar(class = "vec")] rows: Vec<Row>,
  version: u32,
}
```

Encoding a `Table` value produces one postcard payload:

1. The top level is a sequence of length 2: element 0 is `rows`, element 1 is
   `version`.
2. `rows` is a nested sequence:
   - `COL_name` bytes (RLE of `String`)
   - `COL_id` bytes (Delta‑RLE of `u64`)
   - If any `note` present: a pair `(0, COL_note_bytes)` appended
3. `version` is the postcard encoding of `u32`.

A decoder for an older schema without `note` will ignore the `(0, ...)` pair. A
decoder for a newer schema that adds `#[columnar(optional, index = 1)] nick:
Option<String>` will read or default it independently.


## Implementation Notes (Pointers)

- Container wrappers: `columnar/src/wrap.rs`
- Codecs and iterators: `columnar/src/strategy/rle.rs`, `columnar/src/iterable.rs`
- Column types: `columnar/src/column/*.rs`
- Top‑level encode/decode helpers: `columnar/src/lib.rs`,
  `columnar/src/columnar_internal.rs`
- Macro expansion for tables/rows: `columnar_derive/src/**/*`


## Non‑Goals

- No schema negotiation or self‑describing messages. The reader must know the
  Rust types to decode against.
- No cross‑column compression (each column is independent).


## Summary

- Tables are sequences of fields; container fields embed a columnar layout.
- Row fields become independent columns; codecs emit byte strings.
- Optional fields are appended as `(index, bytes)` pairs for
  backward/forward‑compatible evolution.
- Iteration is a decoding optimization; the wire format remains the same.

