# `serde_columnar`

`serde_columnar` is an ergonomic columnar storage encoding crate that offers forward and backward compatibility. 

It allows the contents that need to be serialized and deserialized to be encoded into binary using columnar storage, all by just employing simple macro annotations.

 For more detailed introduction, please refer to this `Notion` link: [Serde-Columnar](https://www.notion.so/loro-dev/Serde-Columnar-Ergonomic-columnar-storage-encoding-crate-7b0c86d6f8d24e4da45a1e2ebd86741c?pvs=4).

## Features 🚀

`serde_columnar` comes with several remarkable features: 

- 🗜️ Utilizes columnar storage in conjunction with various compression strategies to significantly reduce the volume of the encoded content.
- 🔄 Built-in forward and backward compatibility solutions, eliminating the need for maintaining additional version codes.
- 🌳 Supports nested columnar storage.
- 🗃️ Offers additional compression for each column.
- 📦 Supports list and map containers

## How to use

### Install

```shell
cargo add serde_columnar
```

Or edit your `Cargo.toml` and add `serde_columnar` as dependency:

```toml
[dependencies]
serde_columnar = "0.3.0"
```

### Examples

```rust
use serde_columnar::{columnar, from_bytes, to_vec};

#[columnar(vec, ser, de)]                // this struct can be a row of vec-like container
#[derive(Clone, PartialEq)]
struct Data {
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
struct VecStore {
    #[columnar(class = "vec")]          // this field is a vec-like table container
    pub data: Vec<Data>,
    #[columnar(optional, index = 0)]    // table container also supports optional field
		pub other_data: u64
}

let store = VecStore::new(...);
let bytes = serde_columnar::to_vec(&store).unwrap();
let store_from_bytes = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();

```

You can find more examples of `serde_columnar` in `examples` and `tests`.

## Acknowledgements

- [postcard](https://github.com/jamesmunns/postcard): Postcard is a #![no_std] focused serializer and deserializer for Serde. We use it as serializer and deserializer in order to provide VLE and ZigZag encoding.
- [Automerge](https://github.com/automerge/automerge): Automerge is an excellent crdt framework, we reused the code related to RLE Encoding in it.