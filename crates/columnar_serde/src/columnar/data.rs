use std::borrow::Cow;

use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Clone)]
pub enum ColumnData<'c> {
    U64(u64),
    U32(u32),
    String(Cow<'c, str>),
}

impl Serialize for ColumnData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ColumnData::U64(v) => serializer.serialize_u64(*v),
            ColumnData::U32(v) => serializer.serialize_u32(*v),
            ColumnData::String(v) => serializer.serialize_str(v),
        }
    }
}