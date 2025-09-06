# Serde‑Columnar Binary Encoding

This document describes the on‑wire format produced by the `serde_columnar`
crate. It aims to be easy to read and practical to implement against, inspired
by the style of the Automerge binary format spec.

The format is designed for compactness and forward/backward compatibility, with
column‑oriented layout and specialized codecs for common patterns. It is built
on top of the [postcard] serializer for primitive and sequence encodings.

[postcard]: https://docs.rs/postcard/


## Scope and Stability

- This spec describes the current behavior of the `0.3.x` series of
  `serde_columnar` as implemented in this repository.
- The crate is in progress and not yet stable for production use; details may
  change. The compatibility model (optional fields with stable integer indexes)
  is intended to remain.


## High‑Level Model

At a high level, a serialized value is a single [postcard] payload. Within that
payload:

- A “table” (a struct annotated with `#[columnar(ser, de)]`) is encoded as a
  postcard sequence containing each field in declaration order. Fields marked as
  containers (`class="vec"` or `class="map"`) are themselves nested postcard
  sequences in a columnar layout (described below).
- A “row” (a struct annotated with `#[columnar(vec)]` and/or `#[columnar(map)]`)
  is never serialized directly; instead, rows are only encoded as part of a
  container and become one column per field.
- No field names or strategy identifiers are stored on wire. The schema (Rust
  type with its `#[columnar(...)]` attributes) determines how to interpret each
  element.

Postcard provides the basic representation for integers, bytes, sequences, and
tuples. Specifically:

- Integers are varint‑encoded; signed integers use ZigZag.
- A “bytes” value is encoded as a length (varint) followed by that many raw
  octets.
- A sequence is encoded as its length (varint) followed by each element in
  order.

Serde‑columnar uses postcard’s “bytes” to embed codec output, and postcard’s
sequences to arrange columns and optional field mappings.


## Containers and Layout

Two container kinds are supported: list‑like and map‑like.

### Vec‑like containers (`class = "vec"`)

Given a struct `Row` annotated with `#[columnar(vec)]` and a field `data:
Vec<Row>` in a table struct annotated with `#[columnar(ser, de)]`, the value of
`data` is encoded as a sequence of columns, one per field of `Row`:

```
data := SEQ(
  COL_0_bytes,
  COL_1_bytes,
  ...,
  COL_{F-1}_bytes,
  (opt_index, opt_COL_bytes)*        // 0 or more optional fields
)
```

- `F` is the count of non‑optional, non‑skipped fields in `Row`.
- Each `COL_i_bytes` is a postcard “bytes” element produced by the selected
  codec for that column (see Codecs).
- Optional row fields are not placed positionally. Instead, each present
  optional field is appended as a pair `(index: usize, bytes: Vec<u8>)`, where
  `index` is the stable integer specified in the schema
  `#[columnar(optional, index = N)]` and `bytes` is the codec output for that
  optional column.

Decoding:

1. Read the `F` non‑optional columns in order.
2. Read zero or more mapping entries `(index, bytes)` until the sequence ends.
3. For each optional field of `Row`:
   - If present in the mapping: decode that column from its `bytes`.
   - Otherwise: synthesize a column of length `L` (the max length among decoded
     columns) filled with `Default::default()`.
4. Reconstruct rows by zipping the per‑field columns element‑wise.


### Map‑like containers (`class = "map"`)

Given a struct `Row` annotated with `#[columnar(map)]` and a field
`data: Map<K, Row>` in a table struct, the value of `data` is encoded as:

```
data := SEQ(
  KEYS: Vec<K>,                       // postcard Vec<K>
  COL_0_bytes, COL_1_bytes, ..., COL_{F-1}_bytes,
  (opt_index, opt_COL_bytes)*
)
```

The columns correspond to the value type `Row` in the same way as the vec case.
Keys are serialized once up front. During decoding, `len(KEYS)` determines the
expected number of elements per column.

Reconstruction zips the keys with the reconstructed rows to produce the map.


### Tables (top‑level structs)

A table struct annotated with `#[columnar(ser, de)]` is encoded as a postcard
sequence of its fields in declaration order. For each field:

- If `class = "vec"` or `class = "map"`: the field is encoded using the
  container layouts above as a nested sequence value.
- Otherwise: the field is encoded with postcard normally.

Optional table fields are appended after all non‑optional fields as mapping
pairs `(index: usize, bytes: &[u8])`, mirroring the container behavior. Missing
optionals decode to `Default::default()`.


## Columns and Codecs

Each field of a row becomes a column. The schema controls which codec is used by
annotating the field with `#[columnar(strategy = "...")]`. If no strategy is
specified, a generic codec is used.

On wire, every column is carried as a postcard “bytes” element. No strategy tag
is stored; the schema determines how to interpret the bytes.


### Generic (no strategy)

Encodes the raw vector of field values using postcard: `Vec<T>` → `bytes`. This
does not compress and is the fallback for complex types and nested containers
(e.g., a `Vec<Row>` or `Map<K, Row>` nested inside a row).


### RLE (`strategy = "Rle"`)

General run‑length encoding for any `T` that implements Serde, `Clone`, and
`PartialEq`.

Column bytes are a concatenation of runs encoded as postcard values:

- Repeated run: `(len: isize > 0)` followed by a single `value: T`.
- Literal run: `(len: isize < 0)` followed by `-len` values of type `T`.

Decoding appends `len` copies of `value` for repeated runs and replays the
exact sequence for literal runs. A safety limit of `MAX_RLE_COUNT = 1_000_000_000`
is enforced to reject obviously invalid inputs.


### Delta‑RLE (`strategy = "DeltaRle"`)

Optimized for monotonic or slowly changing integers.

Encoding maintains an `absolute` accumulator (starting at `0`). For each value
`v`, compute `delta = v - absolute`, update `absolute = v`, and append `delta`
to an RLE stream of `i128` using the RLE format above. The column bytes are the
underlying RLE bytes.

Decoding reconstructs `absolute += delta` for each delta read from the RLE
decoder and converts back to the target integer type, erroring on overflow.


### Bool‑RLE (`strategy = "BoolRle"`)

Specialized for booleans. The column bytes are a postcard sequence of `usize`
counts. Decoding proceeds as follows:

1. Initialize `value = true` and `count = 0`.
2. For each `n` read:
   - Set `value = !value` (toggle).
   - Emit `n` copies of `value`.

The encoder chooses counts so that the first toggle yields the first run’s
boolean. For example, the sequence `true, true, false, false, false` encodes as
counts `[0, 2, 3]`.

Like RLE, a safety limit guards against pathological inputs.


### Delta‑of‑Delta (`strategy = "DeltaOfDelta"`)

Compact bit‑packed encoding for timestamp‑like series with approximately
constant step. The first element is stored verbatim; subsequent steps encode the
second difference (`Δ² = (v_i - v_{i-1}) - (v_{i-1} - v_{i-2})`).

Column bytes have the following structure:

```
bytes := postcard(Option<i64>)   // head value (Some(first), or None if empty)
         octet                    // bits_used_in_last_byte (1..=8, 8 means a full byte)
         bitstream                // big‑endian packed Δ² codes
```

The bitstream encodes each Δ² using a prefix class and payload:

- `0`                             → Δ² = 0 (1 bit total)
- `10`  + 7 bits unsigned        → Δ² ∈ [−63, 64]
- `110` + 9 bits unsigned        → Δ² ∈ [−255, 256]
- `1110` + 12 bits unsigned      → Δ² ∈ [−2047, 2048]
- `11110` + 21 bits unsigned     → Δ² ∈ [−(2²⁰−1), 2²⁰]
- `11111` + 64 bits unsigned     → Δ² as 64‑bit two’s‑complement

In each non‑zero class, the stored payload is `(Δ² + bias)` where the bias is
63, 255, 2047, or `(2²⁰ − 1)` respectively. Bits are appended MSB‑first,
spanning octet boundaries as needed. The single‑octet `bits_used_in_last_byte`
acts as a tail marker: when the last code ends exactly on a byte boundary, it
is set to `8`.

Decoding reconstructs values by:

1. Reading the head `Option<i64>` to seed `prev` and `prev_delta`.
2. Repeating: read a code; if class `0`, set `prev += prev_delta`; else read the
   payload, unbias to get Δ², update `prev_delta += Δ²`, then `prev +=
   prev_delta`.
3. Stop when there are no more bits in the bitstream.


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
do not know a field’s `index` will ignore it. Decoders that expect a field that
was not sent will use `Default::default()` for that field.

This makes adding, removing, or reordering optional fields forward‑ and
backward‑compatible, as long as each field’s binary representation remains
compatible (e.g., `u32` → `u64` is fine, but changing the meaning of a codec is
not).


## Borrowing and Zero‑Copy

Fields annotated with `#[columnar(borrow)]` are deserialized by borrowing from
the input buffer where possible (e.g., `Cow<'de, str>` and `Cow<'de, [u8]>`).
This does not change the on‑wire format; it affects only how bytes are mapped to
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

- Tables serialize as postcard sequences of fields; container fields embed a
  columnar layout.
- Row fields become independent columns; codecs emit postcard “bytes”.
- Optional fields are appended as `(index, bytes)` pairs, enabling
  backward‑/forward‑compatible evolution.
- Iteration is a decoding optimization; the wire format remains the same.

