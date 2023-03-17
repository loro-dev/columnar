//! ## Container
//!
//! - `#[columnar]` means that some fields (marked by `#[columnar(type = "vec"|"map")]`) of this structure can be serialized and deserialized by columnar encoding
//! - `#[columnar(vec, map)]` means the struct can be a row inside [Vec] or [HashMap]
//! - `#[columnar(ser, de)]` means the struct can be serialized or deserialized or both by columnar encoding
//!
//! ## Field Attributes
//!
//! - `#[columnar(index = 1|2|3...)]`: the id of the field; TODO:
//! - `#[columnar(type = "vec"|"map")]`:
//!   - vec means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=&Value>` `T: FromIterator<Value>`
//!   - map means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=(&K, &Value)>` `T: FromIterator<(K, Value)>`
//! - `#[columnar(strategy = "Rle"|"BoolRle"|"DeltaRle")]`: You can only choose one from the three
//!   - Rle: [`columnar::strategy::rle`]
//!   - BooleanRle
//!   - DeltaRle
//! - `#[columnar(original_type="u32")]`: this attribute is used to tell the columnar encoding the original type of the field, which is used when the field is a number
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
    ColumnAttr,
};
mod columnar_internal;
pub use crate::columnar_internal::{ColumnarDecoder, ColumnarEncoder};
mod row;
pub use row::{KeyRowDe, KeyRowSer, RowDe, RowSer};
mod strategy;
mod wrap;
pub use itertools::{izip, MultiUnzip};
use serde::{Deserialize, Serialize};
pub use strategy::Strategy;
pub use wrap::{ColumnarMap, ColumnarVec};
#[cfg(feature = "compress")]
mod compress;
#[cfg(feature = "compress")]
pub use compress::{compress, decompress, CompressConfig};

pub use postcard::Error as PostcardError;
pub use serde_columnar_derive::*;

#[cfg(feature = "bench")]
extern crate lazy_static;

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
