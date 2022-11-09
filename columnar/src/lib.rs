//! ## Container
//!
//! - `#[columnar]`
//! - `#[columnar(vec, map)]` means the struct can be a rle row inside [Vec] or [HashMap]
//!
//! ## Field Attributes
//!
//! - `#[columnar(index = 1|2|3...)]`: the id of the field; TODO:
//! - `#[columnar(type = "vec"|"map")]`:
//!   - vec means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=&Value>` `T: FromIterator<Value>`
//!   - map means the decorated field T is a container, holds Value and satisfies `&T: IntoIter<Item=(&K, &Value)>` `T: FromIterator<(K, Value)>`
//! - `#[columnar(strategy = "Rle"|"BooleanRle"|"DeltaRle")]`: You can only choose one from the three
//!   - Rle: [`columnar::strategy::rle`]
//!   - BooleanRle
//!   - DeltaRle
//!

mod err;
use std::ops::DerefMut;

pub use err::ColumnarError;
mod column;
pub use column::{Column, ColumnAttr};
mod columnar;
pub use columnar::{ColumnarDecoder, ColumnarEncoder};
mod row;
pub use row::{MapRow, VecRow};
mod strategy;
mod wrap;
use serde::{Deserialize, Serialize};
pub use strategy::Strategy;
pub use wrap::ColumnarVec;
mod serde_impl;

pub use columnar_derive::*;

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
