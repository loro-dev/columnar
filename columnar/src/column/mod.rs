pub mod bool_rle;
pub mod delta_rle;
pub mod rle;
pub mod serde_impl;

use crate::{columnar_internal::ColumnarEncoder, ColumnarDecoder, ColumnarError};
use crate::{BoolRleColumn, DeltaRleColumn, DeltaRleable, RleColumn, Rleable, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::DerefMut;

pub trait ColumnTrait {
    const STRATEGY: Strategy;
    fn attr(&self) -> ColumnAttr;
    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError>;
    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// TODO: remove index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnAttr {
    pub index: Option<usize>,
}

impl ColumnAttr {
    pub(crate) fn empty() -> Self {
        Self { index: None }
    }
}

// TODO: remove this
pub struct ColumnEncoder {
    ser: ColumnarEncoder,
}

impl ColumnEncoder {
    pub(crate) fn new() -> Self {
        Self {
            ser: ColumnarEncoder::new(),
        }
    }

    pub(crate) fn encode<T: ColumnTrait>(mut self, column: &T) -> Result<Vec<u8>, ColumnarError> {
        column.encode(&mut self.ser)?;
        let buf = self.ser.into_bytes();
        Ok(buf)
    }
}

// TODO: remove this
pub struct ColumnDecoder<'b> {
    original_bytes: &'b [u8],
}

impl<'b> ColumnDecoder<'b> {
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Self {
            original_bytes: bytes,
        }
    }

    pub(crate) fn decode<T: ColumnTrait>(&mut self) -> Result<T, ColumnarError> {
        let mut columnar_decoder = ColumnarDecoder::new(self.original_bytes);
        let column = T::decode(&mut columnar_decoder)?;
        Ok(column)
    }
}

impl From<Vec<bool>> for BoolRleColumn {
    fn from(value: Vec<bool>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl<T: DeltaRleable> From<Vec<T>> for DeltaRleColumn<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl<T: Rleable> From<Vec<T>> for RleColumn<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl<T> From<Vec<T>> for GenericColumn<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

#[derive(Debug)]
pub struct GenericColumn<T> {
    pub data: Vec<T>,
    pub attr: ColumnAttr,
}
impl<T> GenericColumn<T> {
    pub fn new(data: Vec<T>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl<T> ColumnTrait for GenericColumn<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    const STRATEGY: Strategy = Strategy::None;

    fn attr(&self) -> ColumnAttr {
        ColumnAttr::empty()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError> {
        self.data.serialize(columnar_encoder.deref_mut())?;
        Ok(())
    }

    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let data = Deserialize::deserialize(columnar_decoder.deref_mut())?;
        Ok(Self {
            data,
            attr: ColumnAttr::empty(),
        })
    }
}
