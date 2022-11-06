use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, collections::HashMap};

use crate::{row::ColumnarVec, Column, ColumnAttr, MapRow, Strategy, VecRow};

type DeltaType = u32;

#[derive(arbitrary::Arbitrary, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Data {
    pub id: DeltaType,
    pub name: String,
}

#[derive(arbitrary::Arbitrary, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VecStore {
    #[serde(serialize_with = "VecRow::serialize_columns")]
    #[serde(deserialize_with = "VecRow::deserialize_columns")]
    // #[serde_as(as = "ColumnarVec<Data>")]
    pub data: Vec<Data>,
    pub id: u8,
}

impl<IT> VecRow<IT> for Data
where
    for<'c> &'c IT: IntoIterator<Item = &'c Self>,
    IT: FromIterator<Self> + Clone,
{
    const FIELD_NUM: usize = 2usize;
    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let column1 = rows.into_iter().map(|row| row.id).collect::<Vec<_>>();
        let column2 = rows
            .into_iter()
            .map(|row| Cow::from(&row.name))
            .collect::<Vec<Cow<'_, str>>>();
        let column1 = Column::new(
            column1,
            ColumnAttr {
                index: 0,
                strategy: Some(Strategy::DeltaRle),
            },
        );
        let column2 = Column::new(
            column2,
            ColumnAttr {
                index: 1,
                strategy: None,
            },
        );
        let mut seq_encoder = ser.serialize_tuple(<Data as VecRow<IT>>::FIELD_NUM)?;
        seq_encoder.serialize_element(&column1)?;
        seq_encoder.serialize_element(&column2)?;
        seq_encoder.end()
    }

    fn deserialize_columns<'de, D>(de: D) -> Result<IT, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (column1, column2): (Column<Cow<DeltaType>>, Column<Cow<String>>) =
            Deserialize::deserialize(de)?;
        let ans = column1
            .data
            .into_iter()
            .zip(column2.data.into_iter())
            .map(|(id, name)| Self {
                id: id.into_owned(),
                name: name.into_owned(),
            })
            .collect();
        Ok(ans)
    }
}

#[derive(arbitrary::Arbitrary, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapStore {
    #[serde(serialize_with = "MapRow::serialize_columns")]
    #[serde(deserialize_with = "MapRow::deserialize_columns")]
    pub data: HashMap<u64, Data>,
}

impl<'de> MapRow<'de> for Data {
    const FIELD_NUM: usize = 2;
    type Key = u64;
    fn serialize_columns<'c, S>(
        rows: &'c HashMap<Self::Key, Self>,
        ser: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (vec_k, (column1, column2)): (Vec<&Self::Key>, (Vec<DeltaType>, Vec<Cow<'c, str>>)) =
            rows.iter()
                .map(|(k, v)| (k, (v.id, Cow::from(&v.name))))
                .unzip();
        let column1 = Column::new(
            column1,
            ColumnAttr {
                index: 0,
                strategy: Some(Strategy::Rle),
            },
        );
        let column2 = Column::new(
            column2,
            ColumnAttr {
                index: 1,
                strategy: None,
            },
        );
        let mut ser_tuple = ser.serialize_tuple(1 + <Data as MapRow>::FIELD_NUM)?;
        ser_tuple.serialize_element(&vec_k)?;
        ser_tuple.serialize_element(&column1)?;
        ser_tuple.serialize_element(&column2)?;
        ser_tuple.end()
    }

    fn deserialize_columns<D>(de: D) -> Result<HashMap<Self::Key, Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (vec_k, column1, column2): (Vec<u64>, Column<DeltaType>, Column<Cow<str>>) =
            Deserialize::deserialize(de)?;
        let ans: Vec<_> = column1
            .data
            .into_iter()
            .zip(column2.data.into_iter())
            .map(|(id, name)| Self {
                id,
                name: name.into_owned(),
            })
            .collect();
        Ok(vec_k.into_iter().zip(ans).collect())
    }
}

#[derive(arbitrary::Arbitrary, Debug, Serialize, Deserialize, PartialEq)]
pub struct NestedStore {
    #[serde(serialize_with = "VecRow::serialize_columns")]
    #[serde(deserialize_with = "VecRow::deserialize_columns")]
    pub stores: Vec<VecStore>, // VecStore ->  data: Vec<Data> id: u8
    pub stores2: HashMap<String, MapStore>,
}

impl<IT> VecRow<IT> for VecStore
where
    for<'c> &'c IT: IntoIterator<Item = &'c Self>,
    IT: FromIterator<Self> + Clone,
{
    const FIELD_NUM: usize = 2;

    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (column1, column2): (Vec<ColumnarVec<Data>>, Vec<u8>) = rows
            .into_iter()
            .map(|row| (ColumnarVec::from_borrowed(&row.data), row.id))
            .unzip();
        let column1 = Column::new(
            column1,
            ColumnAttr {
                index: 0,
                strategy: None,
            },
        );
        let column2 = Column::new(
            column2,
            ColumnAttr {
                index: 1,
                strategy: Some(Strategy::DeltaRle),
            },
        );
        let mut seq_encoder = ser.serialize_tuple(<VecStore as VecRow<IT>>::FIELD_NUM)?;
        seq_encoder.serialize_element(&column1)?;
        seq_encoder.serialize_element(&column2)?;
        seq_encoder.end()
    }

    fn deserialize_columns<'de, D>(de: D) -> Result<IT, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (column1, column2): (Column<ColumnarVec<Data>>, Column<u8>) =
            Deserialize::deserialize(de)?;
        let ans = column1
            .data
            .into_iter()
            .zip(column2.data.into_iter())
            .map(|(data, id)| Self {
                data: data.into(),
                id,
            })
            .collect();
        Ok(ans)
    }
}
