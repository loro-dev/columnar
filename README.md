# columnar

## Introduction

`serde_columnar` is a crate that provides columnar storage for **List** and **Map** with compressible serialization and deserialization capabilities.

Columnar storage is very useful when you want to compress serialized data and you know that one or more fields of consecutive structs in the array have the same or equal difference values.

For example, you want to store this array:

```plain
[{a: 1, b: 1}, {a: 1, b: 2}, {a: 1, b: 3}, ...]
```

After columnar storage, it can be stored as:

```plain
a: [1, 1, 1,...] ---Rle---> [N, 1]
b: [1, 2, 3,...] ---DeltaRle---> [N, 1] (each value is 1 greater than the previous one)
```

## Usage

```rust ignore
type ID = u64;
#[columnar(vec, ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    num: u32,
    #[columnar(strategy = "DeltaRle", original_type = "u64")]
    id: ID,
    #[columnar(strategy = "Rle")]
    gender: String,
    #[columnar(strategy = "BoolRle")]
    married: bool
}

#[columnar(ser, de)]
#[derive(Debug, Serialize, Deserialize)]
pub struct VecStore {
    #[columnar(type = "vec")]
    pub data: Vec<Data>
}


let store = VecStore::new(...);
let bytes = serde_columnar::to_vec(&store).unwrap();
let store = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();

```

## More Details

### Container

- `#[columnar]` means that some fields (marked by `#[columnar(type = "vec"|"map")]`) of this structure can be serialized and deserialized by columnar encoding
- `#[columnar(vec, map)]` means the struct can be a row inside `Vec-like` or `Map-like`
- `#[columnar(ser, de)]` means the struct can be serialized or deserialized or both by columnar encoding

### Field Attributes

- `#[columnar(type = "vec"|"map")]`:
  - vec means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=&Value>` `T: FromIterator<Value>`
  - map means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=(&K, &Value)>` `T: FromIterator<(K, Value)>`
- `#[columnar(strategy = "Rle"|"BoolRle"|"DeltaRle")]`: You can only choose one from the three
  - Rle [crate::strategy::AnyRleEncoder]
  - BoolRle [crate::strategy::BoolRleEncoder]
  - DeltaRle [crate::strategy::DeltaRleEncoder]
- `#[columnar(original_type="u32")]`: this attribute is used to tell the columnar encoding the original type of the field, which is used when the field is a number
- `#[columnar(skip)]`: the same as the [skip](https://serde.rs/field-attrs.html#skip) attribute in serde

### Compress (enable `compress` feature)

- `#[columnar(compress)]`: compress the columnar encoded bytes by
  [default settings](https://docs.rs/flate2/latest/flate2/struct.Compression.html#impl-Default) of Deflate algorithm.

- more compress options:
  - `#[columnar(compress(min_size=N))]`: compress the columnar encoded bytes when the size of the bytes is larger than N, **default N is 256**.
  - `#[columnar(compress(level=N))]`: compress the columnar encoded bytes by Deflate algorithm with level N, N is in [0, 9], default N is 6,
    0 is no compression, 9 is the best compression. See [flate2](https://docs.rs/flate2/latest/flate2/struct.Compression.html#) for more details.
  - `#[columnar(compress(method="fast"|"best"|"default"))]`: compress the columnar encoded bytes by Deflate algorithm with method "fast", "best" or "default",
    this attribute is equivalent to `#[columnar(compress(level=1|9|6))]`.
  - Note: `level` and `method` can not be used at the same time.
