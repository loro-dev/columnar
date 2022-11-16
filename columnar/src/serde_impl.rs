use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    column::{ColumnDecoder, ColumnEncoder},
    compress::decompress,
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
        let bytes = columnar.encode(self).map_err(|e| {
            // println!("Column Serialize Error: {:?}", e);
            serde::ser::Error::custom(e.to_string())
        })?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de, T> Deserialize<'de> for Column<T>
where
    T: Clone + PartialEq + for<'d> Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor<T>(PhantomData<T>);
        impl<'de, T> serde::de::Visitor<'de> for ColumnVisitor<T>
        where
            T: Clone + PartialEq + for<'d> Deserialize<'d>,
        {
            type Value = Column<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a column")
            }
            fn visit_borrowed_bytes<E>(self, bytes: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if bytes.is_empty() {
                    return Err(E::custom("invalid column bytes, bytes len < 1"));
                }
                let compress_flag = bytes[0];
                let column = if compress_flag == 1 {
                    let buf = decompress(&bytes[1..]).map_err(|e| {
                        // println!("Column Decompress Error: {:?}", e);
                        serde::de::Error::custom(e.to_string())
                    })?;
                    let mut decoder = ColumnDecoder::new(&buf);
                    decoder.decode().map_err(|e| {
                        // println!("Column Deserialize Error: {:?}", e);
                        serde::de::Error::custom(e.to_string())
                    })?
                } else {
                    let mut decoder = ColumnDecoder::new(&bytes[1..]);
                    decoder.decode().map_err(|e| {
                        // println!("Column Deserialize Error: {:?}", e);
                        serde::de::Error::custom(e.to_string())
                    })?
                };
                Ok(column)
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor(PhantomData))
    }
}
