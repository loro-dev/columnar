use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    column::{ColumnDecoder, ColumnEncoder},
    Column,
};

impl<T> Serialize for Column<T>
where
    T: Clone + Serialize + PartialEq,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let columnar = ColumnEncoder::new();
        let bytes = columnar
            .encode(self)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?;
        serializer.serialize_bytes(bytes.as_slice())
    }
}

impl<'de, T> Deserialize<'de> for Column<T>
where
    T: Clone + PartialEq + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor<T>(PhantomData<T>);
        impl<'de, T> serde::de::Visitor<'de> for ColumnVisitor<T>
        where
            T: Clone + PartialEq + Deserialize<'de>,
        {
            type Value = Column<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a column")
            }
            fn visit_borrowed_bytes<E>(self, bytes: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut decoder = ColumnDecoder::new(bytes);
                let column = decoder
                    .decode()
                    .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                Ok(column)
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor(PhantomData))
    }
}
