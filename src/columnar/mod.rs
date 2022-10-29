mod data;
use std::{collections::HashMap, marker::PhantomData, ops::Deref};

pub use data::CellData;
mod attr;
pub use attr::{ColumnAttr, Strategy};
use serde::{Deserialize, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

use crate::{columnar_impl::ser::ColumnEncoder, de::ColumnDecoder, ColumnarError};

pub trait ColumnOriented: Sized {
    fn get_columns<'c>(&'c self) -> Columns<'c>;
    fn from_columns(columns: Columns) -> Result<Self, ColumnarError>;
}

pub trait Row: Serialize + Sized {
    fn get_attrs() -> Vec<ColumnAttr>;
    fn get_cells_data<'a: 'c, 'c>(&'a self) -> Vec<CellData<'c>>;
    fn from_cells_data(cells_data: Vec<CellData>) -> Result<Self, ColumnarError>;
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

impl<T> ColumnOriented for Vec<T>
where
    T: Row,
{
    fn get_columns<'c>(&'c self) -> Columns<'c> {
        let attrs = T::get_attrs();
        // N*len(attrs)
        let data = self
            .iter()
            .map(|row| row.get_cells_data())
            .collect::<Vec<_>>();
        let columns_data = transpose(data);
        Columns::from_rows(columns_data, attrs)
    }
    fn from_columns(columns: Columns) -> Result<Self, ColumnarError> {
        let cells_data: Vec<Vec<CellData>> = columns.to_cell_data();
        let rows_data = transpose(cells_data);
        let s: Vec<T> = rows_data
            .into_iter()
            .map(|row| T::from_cells_data(row).unwrap())
            .collect();
        Ok(s)
    }
}

// 一个Row以列式排列的数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct Column<'c>(pub(crate) Vec<CellData<'c>>, pub(crate) ColumnAttr);

#[derive(Debug, Clone, PartialEq)]
pub struct Columns<'c>(Vec<Column<'c>>);

impl<'c> Deref for Columns<'c> {
    type Target = Vec<Column<'c>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'c> Columns<'c> {
    pub fn from_rows(data: Vec<Vec<CellData<'c>>>, attrs: Vec<ColumnAttr>) -> Self {
        assert!(
            data.len() == attrs.len(),
            "{}",
            format!(
                "data({:?}) and attrs({:?}) must have the same length",
                data.len(),
                attrs.len()
            )
        );
        let mut columns = Vec::with_capacity(data.len());
        for (columnar, attr) in data.into_iter().zip(attrs) {
            columns.push(Column(columnar, attr));
        }
        Self(columns)
    }

    pub(crate) fn to_cell_data(self) -> Vec<Vec<CellData<'c>>> {
        self.0.into_iter().map(|c| c.0).collect()
    }
}

impl<'c, T> From<&'c T> for Columns<'c>
where
    T: ColumnOriented,
{
    fn from(obj: &'c T) -> Self {
        obj.get_columns()
    }
}

impl<T> SerializeAs<Vec<T>> for Columns<'_>
where
    T: Row,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let columns = Columns::from(source);
        columns.serialize(serializer)
    }
}

impl Serialize for Columns<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut columnar = ColumnEncoder::new();
        columnar.encode(&self).unwrap();
        let bytes = columnar.finish();
        serializer.serialize_bytes(bytes.as_slice())
    }
}

impl<'de, T: Deserialize<'de> + Row> DeserializeAs<'de, Vec<T>> for Columns<'de> {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let columns: Columns = Deserialize::deserialize(deserializer)?;
        let rows = Vec::<T>::from_columns(columns).unwrap();
        Ok(rows)
    }
}

impl<'de> Deserialize<'de> for Columns<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ColumnsVisitor;
        impl<'de> serde::de::Visitor<'de> for ColumnsVisitor {
            type Value = Columns<'de>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a columnar data")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut decoder = ColumnDecoder::new(v);
                let columns = decoder.decode_columns();
                Ok(columns)
            }
        }
        todo!()
    }
}
