use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, collections::HashMap};

use columnar::{columnar, to_vec, Column, ColumnAttr, ColumnarVec, MapRow, Strategy, VecRow};

type DeltaType = u32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Data {
    pub id: DeltaType,
    pub name: String,
}

#[columnar]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VecStore {
    #[columnar(type = "vec")]
    pub data: Vec<Data>,
    pub id: u8,
}

impl<IT> VecRow<IT> for Data
where
    for<'c> &'c IT: IntoIterator<Item = &'c Self>,
    IT: FromIterator<Self> + Clone,
{
    const FIELD_NUM: usize = 2;
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
        let (column1, column2): (Column<DeltaType>, Column<Cow<str>>) =
            Deserialize::deserialize(de)?;
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

#[test]
fn test() {
    let mut vec_store = VecStore {
        data: vec![Data {
            id: 1,
            name: "hello".to_string(),
        }],
        id: 1,
    };
    let buf = to_vec(&vec_store).unwrap();
    println!("{:?}", buf);
}
