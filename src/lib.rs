mod columnar;
pub use columnar::{CellData, Column, ColumnAttr, ColumnOriented, Columns, Row, Strategy};
mod columnar_impl;
pub use columnar_impl::{de, ser};
use ser::ColumnarSerializer;
mod err;
pub use err::ColumnarError;
use serde::Serialize;

pub fn columnar_encode<T>(obj: T) -> Vec<u8>
where
    T: Serialize,
{
    let mut serializer = ColumnarSerializer::new();
    obj.serialize(&mut serializer).unwrap();
    serializer.to_bytes()
}

pub fn columnar_decode<'de, T>(bytes: &'de [u8]) -> T
where
    T: serde::Deserialize<'de>,
{
    let mut de = de::ColumnarDeserializer::new(bytes);
    T::deserialize(&mut de).unwrap()
}
