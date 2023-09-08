//! # Introduction
//!
//! `serde_columnar` is a crate that provides columnar storage for **List** and **Map** with compressible serialization and deserialization capabilities.
//!
//! Columnar storage is very useful when you want to compress serialized data and you know that one or more fields of consecutive structs in the array have the same or equal difference values.
//!
//! For example, you want to store this array:
//!
//! ```
//! [{a: 1, b: 1}, {a: 1, b: 2}, {a: 1, b: 3}, ...]
//! ```
//! After columnar storage, it can be stored as:
//!
//! ```
//! a: [1, 1, 1,...] ---Rle---> [N, 1]
//! b: [1, 2, 3,...] ---DeltaRle---> [N, 1] (each value is 1 greater than the previous one)
//! ```
//!
//! # Usage
//!
//! ```rust ignore
//! type ID = u64;
//! #[columnar(vec, ser, de)]
//! #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
//! pub struct Data {
//!     #[columnar(strategy = "Rle")]
//!     num: u32,
//!     #[columnar(strategy = "DeltaRle", original_type = "u64")]
//!     id: ID,
//!     #[columnar(strategy = "Rle")]
//!     gender: String,
//!     #[columnar(strategy = "BoolRle")]
//!     married: bool
//! }
//!
//! #[columnar]
//! #[derive(Debug, Serialize, Deserialize)]
//! pub struct VecStore {
//!     #[columnar(type = "vec")]
//!     pub data: Vec<Data>
//! }
//!
//!
//! let store = VecStore::new(...);
//! let bytes = serde_columnar::to_vec(&store).unwrap();
//! let store = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();
//!
//! ```
//!
//! # More Details
//!
//! ## Container
//!
//! - `#[columnar]` means that some fields (marked by `#[columnar(type = "vec"|"map")]`) of this structure can be serialized and deserialized by columnar encoding
//! - `#[columnar(vec, map)]` means the struct can be a row inside `Vec-like` or `Map-like`
//! - `#[columnar(ser, de)]` means the struct can be serialized or deserialized or both by columnar encoding
//!
//! ## Field Attributes
//!
//! - `#[columnar(type = "vec"|"map")]`:
//!   - vec means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=&Value>` `T: FromIterator<Value>`
//!   - map means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=(&K, &Value)>` `T: FromIterator<(K, Value)>`
//! - `#[columnar(strategy = "Rle"|"BoolRle"|"DeltaRle")]`: You can only choose one from the three
//!   - Rle [crate::strategy::AnyRleEncoder]
//!   - BoolRle [crate::strategy::BoolRleEncoder]
//!   - DeltaRle [crate::strategy::DeltaRleEncoder]
//! - `#[columnar(original_type="u32")]`: this attribute is used to tell the columnar encoding the original type of the field, which is used when the field is a number
//! - `#[columnar(skip)]`: the same as the [skip](https://serde.rs/field-attrs.html#skip) attribute in serde
//!
//! ## Compress
//!
//! - `#[columnar(compress)]`: compress the columnar encoded bytes by
//! [default settings](https://docs.rs/flate2/latest/flate2/struct.Compression.html#impl-Default) of Deflate algorithm.
//!
//! - more compress options:
//!   - `#[columnar(compress(min_size=N))]`: compress the columnar encoded bytes when the size of the bytes is larger than N, **default N is 256**.
//!   - `#[columnar(compress(level=N))]`: compress the columnar encoded bytes by Deflate algorithm with level N, N is in [0, 9], default N is 6,
//! 0 is no compression, 9 is the best compression. See [flate2](https://docs.rs/flate2/latest/flate2/struct.Compression.html#) for more details.
//!   - `#[columnar(compress(method="fast"|"best"|"default"))]`: compress the columnar encoded bytes by Deflate algorithm with method "fast", "best" or "default",
//!       this attribute is equivalent to `#[columnar(compress(level=1|9|6))]`.
//!   - Note: `level` and `method` can not be used at the same time.
//!      

mod err;
use std::ops::DerefMut;

pub use err::ColumnarError;
mod column;
pub use column::{
    bool_rle::BoolRleColumn,
    delta_rle::{DeltaRleColumn, DeltaRleable},
    rle::{RleColumn, Rleable},
    ColumnAttr, ColumnDecoder, ColumnEncoder, ColumnTrait, GenericColumn,
};
mod columnar_internal;
pub use columnar_internal::{ColumnarDecoder, ColumnarEncoder};
pub mod iterable;
mod row;
pub use itertools::{izip, Itertools, MultiUnzip};
pub use row::{KeyRowDe, KeyRowSer, RowDe, RowSer};
use serde::{Deserialize, Serialize};
mod strategy;
pub use strategy::{
    AnyRleDecoder, AnyRleEncoder, BoolRleDecoder, BoolRleEncoder, DeltaRleDecoder, DeltaRleEncoder,
    Strategy,
};
mod wrap;
pub use wrap::{ColumnarMap, ColumnarVec};
#[cfg(feature = "compress")]
mod compress;
#[cfg(feature = "compress")]
pub use compress::{compress, decompress, CompressConfig};

pub use postcard::Error as PostcardError;
pub use serde_columnar_derive::*;

#[cfg(feature = "bench")]
extern crate lazy_static;

#[cfg(feature = "analyze")]
mod analyze;
#[cfg(feature = "analyze")]
pub use analyze::{AnalyzeResult, AnalyzeResults, FieldAnalyze};
#[cfg(feature = "analyze")]
pub use serde_columnar_derive::FieldAnalyze;

pub fn to_vec<T: Serialize>(val: &T) -> Result<Vec<u8>, ColumnarError> {
    let mut encoder = ColumnarEncoder::new();
    val.serialize(encoder.deref_mut())
        .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))?;
    Ok(encoder.into_bytes())
}

pub fn from_bytes<'de, 'a: 'de, T: Deserialize<'de>>(bytes: &'a [u8]) -> Result<T, ColumnarError> {
    let mut decoder = ColumnarDecoder::<'de>::new(bytes);
    T::deserialize(decoder.deref_mut())
        .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))
}

pub fn iter_from_bytes<'de, T: iterable::TableIter<'de>>(
    bytes: &'de [u8],
) -> Result<T::Iter, ColumnarError> {
    let mut decoder = ColumnarDecoder::<'de>::new(bytes);
    T::Iter::deserialize(decoder.deref_mut())
        .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))
}
