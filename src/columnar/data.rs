use std::borrow::Cow;

use serde::{Serialize, Serializer};

use crate::{ColumnarError, Columns};

#[derive(Debug, PartialEq, Clone)]
pub enum CellData<'c> {
    U64(u64),
    I64(i64),
    String(Cow<'c, str>),
    Bool(bool),
    Bytes(Cow<'c, [u8]>),
    Columns(Columns<'c>),
}

impl Serialize for CellData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CellData::U64(v) => serializer.serialize_u64(*v),
            CellData::I64(v) => serializer.serialize_i64(*v),
            CellData::String(v) => serializer.serialize_str(v),
            CellData::Bool(v) => serializer.serialize_bool(*v),
            CellData::Bytes(v) => serializer.serialize_bytes(v),
            CellData::Columns(v) => v.serialize(serializer),
        }
    }
}

// TODO: borrow from

impl<'c> TryFrom<CellData<'c>> for Columns<'c> {
    type Error = ColumnarError;

    fn try_from(value: CellData<'c>) -> Result<Self, Self::Error> {
        match value {
            CellData::Columns(v) => Ok(v),
            _ => Err(ColumnarError::InvalidDataType),
        }
    }
}

macro_rules! impl_try_from_cell_data {
    ($t:ty, $variant:ident) => {
        impl TryFrom<CellData<'_>> for $t {
            type Error = ColumnarError;

            fn try_from(value: CellData<'_>) -> Result<Self, Self::Error> {
                match value {
                    CellData::$variant(v) => Ok(v),
                    _ => Err(ColumnarError::InvalidDataType),
                }
            }
        }
    };
}

impl_try_from_cell_data!(u64, U64);
impl_try_from_cell_data!(i64, I64);
impl_try_from_cell_data!(bool, Bool);

macro_rules! impl_try_from_cow_cell_data {
    ($t:ty, $variant:ident) => {
        impl TryFrom<CellData<'_>> for $t {
            type Error = ColumnarError;

            fn try_from(value: CellData<'_>) -> Result<Self, Self::Error> {
                match value {
                    CellData::$variant(v) => Ok(v.into_owned()),
                    _ => Err(ColumnarError::InvalidDataType),
                }
            }
        }
    };
}

impl_try_from_cow_cell_data!(String, String);
impl_try_from_cow_cell_data!(Vec<u8>, Bytes);
