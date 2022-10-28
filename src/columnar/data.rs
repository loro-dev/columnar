use std::borrow::Cow;

use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Clone)]
pub enum CellData<'c> {
    U64(u64),
    I64(i64),
    String(Cow<'c, str>),
    Bool(bool),
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
        }
    }
}
