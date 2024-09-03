pub mod bool_rle;
pub mod delta_of_delta;
pub mod delta_rle;
pub mod rle;
pub mod serde_impl;

use crate::{columnar_internal::ColumnarEncoder, ColumnarDecoder, ColumnarError};
use crate::{
    BoolRleColumn, DeltaOfDeltaColumn, DeltaRleColumn, DeltaRleable, RleColumn, Rleable, Strategy,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::DerefMut;

pub trait ColumnTrait {
    const STRATEGY: Strategy;
    fn attr(&self) -> ColumnAttr;
    fn encode(&self) -> Result<Vec<u8>, ColumnarError>;
    fn decode(bytes: &[u8]) -> Result<Self, ColumnarError>
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

impl From<Vec<bool>> for BoolRleColumn {
    fn from(value: Vec<bool>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl From<Vec<i64>> for DeltaOfDeltaColumn {
    fn from(value: Vec<i64>) -> Self {
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

    fn encode(&self) -> Result<Vec<u8>, ColumnarError> {
        let mut encoder = ColumnarEncoder::new();
        self.data.serialize(encoder.deref_mut())?;
        Ok(encoder.into_bytes())
    }

    fn decode(bytes: &[u8]) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut columnar_decoder = ColumnarDecoder::new(bytes);
        let data = Deserialize::deserialize(columnar_decoder.deref_mut())?;
        Ok(Self {
            data,
            attr: ColumnAttr::empty(),
        })
    }
}
