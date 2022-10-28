use std::borrow::Cow;

use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Clone)]
pub enum ColumnData<'c> {
    U64(u64),
    I64(i64),
    String(Cow<'c, str>),
    Bool(bool)
}

impl Serialize for ColumnData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ColumnData::U64(v) => serializer.serialize_u64(*v),
            ColumnData::I64(v) => serializer.serialize_i64(*v),
            ColumnData::String(v) => serializer.serialize_str(v),
            ColumnData::Bool(v) => serializer.serialize_bool(*v)
        }
    }
}
