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
use serde::{Deserialize, Serialize};
pub use strategy::Strategy;
mod serde_impl;

// #[cfg(feature = "fuzzing")]
pub mod fuzz;

pub fn to_vec<T: Serialize>(val: &T) -> Result<Vec<u8>, ColumnarError> {
    let mut encoder = ColumnarEncoder::new();
    val.serialize(encoder.deref_mut())
        .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))?;
    Ok(encoder.into_bytes())
}

pub fn from_bytes<'de, 'a: 'de, T: Deserialize<'de>>(bytes: &'a [u8]) -> Result<T, ColumnarError> {
    let mut decoder = ColumnarDecoder::<'de>::new(&bytes);
    T::deserialize(decoder.deref_mut())
        .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))
}
