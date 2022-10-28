mod data;
use std::ops::Deref;

pub use data::CellData;
mod attr;
pub use attr::{ColumnAttr, Strategy};
use serde::{Serialize, Serializer};
use serde_with::SerializeAs;

use crate::columnar_impl::ser::ColumnEncoder;

pub trait ColumnOriented {
    fn get_columns<'c>(&'c self) -> Columns<'c>;
}

pub trait Row {
    fn get_attrs() -> Vec<ColumnAttr>;
    fn get_cells_data<'a: 'c, 'c>(&'a self) -> Vec<CellData<'c>>;
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
}

// 一个Row以列式排列的数据结构
#[derive(Debug)]
pub struct Column<'c>(pub(crate) Vec<CellData<'c>>, pub(crate) ColumnAttr);

#[derive(Debug)]
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
}

impl<'c, T> From<&'c T> for Columns<'c>
where
    T: ColumnOriented + 'c,
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
