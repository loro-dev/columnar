use arbitrary::Arbitrary;
use columnar::{columnar, Column, ColumnAttr, ColumnarVec, Strategy, VecRow};
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use std::collections::HashMap;

#[columnar(vec, map)]
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u32,
    name: String,
}

#[columnar]
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq)]
pub struct VecStore {
    #[columnar(type = "vec")]
    data: Vec<Data>,
    #[columnar(strategy = "DeltaRle")]
    id: u64,
}

#[columnar]
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq)]
pub struct MapStore {
    #[columnar(type = "map")]
    data: HashMap<u64, Data>,
    id: u64,
}

#[columnar]
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq)]
pub struct NestedStore {
    #[columnar(type = "vec")]
    stores: Vec<VecStore>,
}

// impl VecRow for VecStore manually until we can derive it by macro
impl<IT> VecRow<IT> for VecStore
where
    IT: FromIterator<Self> + Clone,
    for<'c> &'c IT: IntoIterator<Item = &'c Self>,
{
    const FIELD_NUM: usize = 2;

    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (column1, column2): (Vec<ColumnarVec<Data>>, Vec<u64>) = rows
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
        let (column1, column2): (Column<ColumnarVec<Data>>, Column<u64>) =
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
