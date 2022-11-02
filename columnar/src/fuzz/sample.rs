use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, ops::DerefMut};

use crate::{Column, ColumnAttr, ColumnarDecoder, ColumnarEncoder, Row, Strategy};

#[derive(arbitrary::Arbitrary, Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    id: u64,
    name: String,
}

#[derive(arbitrary::Arbitrary, Debug, Serialize, Deserialize, PartialEq)]
pub struct Store {
    #[serde(serialize_with = "Data::serialize_vec_as_columns")]
    #[serde(deserialize_with = "Data::deserialize_columns_to_vec")]
    data: Vec<Data>,
}

impl Row for Data {
    const FIELD_NUM: usize = 2;
    fn serialize_vec_as_columns<'c, S>(rows: &'c Vec<Self>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let column1 = rows.iter().map(|row| row.id).collect::<Vec<u64>>();
        let column2 = rows
            .iter()
            .map(|row| Cow::from(&row.name))
            .collect::<Vec<Cow<'c, str>>>();
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
        let mut seq_encoder = ser.serialize_tuple(Self::FIELD_NUM)?;
        seq_encoder.serialize_element(&column1)?;
        seq_encoder.serialize_element(&column2)?;
        seq_encoder.end()
    }

    fn deserialize_columns_to_vec<'de, D>(de: D) -> Result<Vec<Self>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (column1, column2): (Column<u64>, Column<Cow<str>>) = Deserialize::deserialize(de)?;
        let ans = column1
            .data
            .into_iter()
            .zip(column2.data.into_iter())
            .map(|(id, name)| Self {
                id,
                name: name.into_owned(),
            })
            .collect();
        Ok(ans)
    }
}
