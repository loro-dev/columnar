# `serde_columnar`

`serde_columnar` is an ergonomic columnar storage encoding crate that offers forward and backward compatibility.

It allows the contents that need to be serialized and deserialized to be encoded into binary using columnar storage, all by just employing simple macro annotations.

For more detailed introduction, please refer to this `Notion` link: [Serde-Columnar](https://www.notion.so/loro-dev/Serde-Columnar-Ergonomic-columnar-storage-encoding-crate-7b0c86d6f8d24e4da45a1e2ebd86741c?pvs=4).

## 🚧 This crate is in progress and not stable, should not be used in production environments

## Features 🚀

`serde_columnar` comes with several remarkable features:

- 🗜️ Utilizes columnar storage in conjunction with various compression strategies to significantly reduce the size of the encoded content.
- 🔄 Built-in forward and backward compatibility solutions, eliminating the need for maintaining additional version codes.
- 🌳 Supports nested columnar storage.
- 🗃️ Offers additional compression for each column.
- 📦 Supports list and map containers
- 🔄 Supports deserialization using iterator format.

## How to use

### Install

```shell
cargo add serde_columnar
```

Or edit your `Cargo.toml` and add `serde_columnar` as dependency:

```toml
[dependencies]
serde_columnar = "0.3.2"
```

### Container Attribute

- `vec`:
  - Declare this struct will be rows of a vec-like container
  - Automatically derive [`RowSer`](https://docs.rs/serde_columnar/latest/serde_columnar/trait.RowSer.html) trait if set `ser` at the same time
  - Automatically derive [`RowDe`](https://docs.rs/serde_columnar/latest/serde_columnar/trait.RowDe.html) trait if set `de` at the same time
- `map`:
  - Declare this struct will be rows of a map-like container
  - Automatically derive [`KeyRowSer`](https://docs.rs/serde_columnar/latest/serde_columnar/trait.KeyRowSer.html) trait if set `ser` at the same time
  - Automatically derive [`KeyRowDe`](https://docs.rs/serde_columnar/latest/serde_columnar/trait.KeyRowDe.html) trait if set `de` at the same time
- `ser`:
  - Automatically derive `Serialize` trait for this struct
- `de`:
  - Automatically derive `Deserialize` trait for this struct
- `iterable`:
  - Declare this struct will be iterable
  - Only available for `row` struct
  - [Iterable](https://github.com/loro-dev/columnar#Iterable) for more details

### Field Attribute

- `strategy`:
  - The columnar compression strategy applied to this field.
  - Optional value: `Rle`/`DeltaRle`/`BoolRle`.
  - Only available for `row` struct.
- `class`:
  - Declare this field is a container for rows. The field's type is usually `Vec` or `HashMap` and their variants.
  - Optional value: `vec` or `map`.
  - Only available for `table` struct.
- `skip`:
  - Same as [`#[serde(skip)]`](https://serde.rs/field-attrs.html#skip), do not serialize or deserialize this field.
- `borrow`:
  - Same as [`#[serde(borrow)]`](https://serde.rs/field-attrs.html#borrow), borrow data for this field from the deserializer by using zero-copy deserialization.
  - use `#[columnar(borrow="'a + 'b")]` to specify explicitly which lifetimes should be borrowed.
  - Only available for `table` struct for now.
- `iter`:
  - Declare the iterable row type when deserializing using iter mode.
  - Only available for field marked `class`.
  - Only available for `class="vec"`.
  - Unavailable with `compress`.
- `optional` & `index`:
  - In order to achieve forward and backward compatibility, some fields that may change can be marked as `optional`.
  - And in order to avoid the possibility of errors in the future, such as change the order of optional fields, it is necessary to mark the `index`.
  - All `optional` fields must be after other fields.
  - The `index` is the unique identifier of the optional field, which will be encoded into the result. If the corresponding identifier cannot be found during deserialization, `Default` will be used.
  - `optional` fields can be added or removed in future versions. The compatibility premise is that the field type of the same index does not change or the encoding format is compatible (such as changing `u32` to `u64`).
- `compress`:
  - **This attribute needs to enable the `compress` feature**
  - This attribute is whether compress the columnar encoded bytes by default settings of Deflate algorithm.
  - `#[columnar(compress(min_size=N))]`: compress the columnar encoded bytes when the size of the bytes is larger than N, **default N is 256**.
  - `#[columnar(compress(level=N))]`: compress the columnar encoded bytes by Deflate algorithm with level N, N is in [0, 9], default N is 6, 0 is no compression, and 9 is the best compression. See [flate2](https://docs.rs/flate2/latest/flate2/struct.Compression.html#) for more details.
  - `#[columnar(compress(method="fast"|"best"|"default"))]`: compress the columnar encoded bytes by Deflate algorithm with method "fast", "best" or "default", this attribute is equivalent to `#[columnar(compress(level=1|9|6))]`.
  - **Note: `level` and `method` can not be used at the same time.**
  - Only available for `row` struct.

### Examples

```rust
use serde_columnar::{columnar, from_bytes, to_vec};

#[columnar(vec, ser, de)]                // this struct can be a row of vec-like container
struct RowStruct {
    name: String,
    #[columnar(strategy = "DeltaRle")]   // this field will be encoded by `DeltaRle`
    id: u64,
    #[columnar(strategy = "Rle")]        // this field will be encoded by `Rle`
    gender: String,
    #[columnar(strategy = "BoolRle")]    // this field will be encoded by `BoolRle`
    married: bool
    #[columnar(optional, index = 0)]     // This field is optional, which means that this field can be added in this version or deleted in a future version
    future: String
}

#[columnar(ser, de)]                    // derive `Serialize` and `Deserialize`
struct TableStruct<'a> {
    #[columnar(class = "vec")]          // this field is a vec-like table container
    pub data: Vec<RowStruct>,
    #[columnar(borrow)]                 // the same as `#[serde(borrow)]`
    pub text: Cow<'a, str>
    #[columnar(skip)]                   // the same as `#[serde(skip)]`
    pub ignore: u8
    #[columnar(optional, index = 0)]    // table container also supports optional field
    pub other_data: u64

}

let table = TableStruct::new(...);
let bytes = serde_columnar::to_vec(&table).unwrap();
let table_from_bytes = serde_columnar::from_bytes::<TableStruct>(&bytes).unwrap();

```

You can find more examples of `serde_columnar` in `examples` and `tests`.

### Iterable

When we use columnar for compression encoding, there is a premise that the field is iterable. So we can completely borrow the encoded bytes to obtain all the data in the form of iterator during deserialization without directly allocating the memory of all the data. This implementation can also be achieved completely through macros.

To use iter mode when deserializing, you only need to do 3 things:

1. mark all row struct with `iterable`
2. mark the field of row container with `iter="..."`
3. use `serde_columnar::iter_from_bytes` to deserialize

```rust
#[columnar(vec, ser, de, iterable)]
struct Row{
  #[columnar(strategy="Rle")]
  rle: String
  #[columnar(strategy="DeltaRle")]
  delta_rle: u64
  other: u8
}

#[columnar(ser, de)]
struct Table{
  #[columnar(class="vec", iter="Row")]
  vec: Vec<Row>,
  other: u8
}

let table = Table::new(...);
let bytes = serde_columnar::to_vec(&table).unwrap();
let table_iter = serde_columnar::iter_from_bytes::<Table>(&bytes).unwrap();

```

## Acknowledgements

- [serde](https://github.com/serde-rs/serde): Serialization framework for Rust.
- [postcard](https://github.com/jamesmunns/postcard): Postcard is a #![no_std] focused serializer and deserializer for Serde. We use it as serializer and deserializer in order to provide VLE and ZigZag encoding.
- [Automerge](https://github.com/automerge/automerge): Automerge is an excellent crdt framework, we reused the code related to RLE Encoding in it.
