use columnar::columnar;
use serde;

#[columnar]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VecStore {
    #[columnar]
    pub id: u8,
}

// impl VecRow for Data {
//     type Input = Vec<Self>;
//     const FIELD_NUM: usize = 2;
//     fn serialize_columns<'c, S>(rows: &'c Vec<Self>, ser: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let column1 = rows.iter().map(|row| row.id).collect::<Vec<DeltaType>>();
//         let column2 = rows
//             .iter()
//             .map(|row| Cow::from(&row.name))
//             .collect::<Vec<Cow<'c, str>>>();
//         let column1 = Column::new(
//             column1,
//             ColumnAttr {
//                 index: 0,
//                 strategy: Some(Strategy::DeltaRle),
//             },
//         );
//         let column2 = Column::new(
//             column2,
//             ColumnAttr {
//                 index: 1,
//                 strategy: None,
//             },
//         );
//         let mut seq_encoder = ser.serialize_tuple(<Data as VecRow>::FIELD_NUM)?;
//         seq_encoder.serialize_element(&column1)?;
//         seq_encoder.serialize_element(&column2)?;
//         seq_encoder.end()
//     }

//     fn deserialize_columns<'de, D>(de: D) -> Result<Vec<Self>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let (column1, column2): (Column<DeltaType>, Column<Cow<str>>) =
//             Deserialize::deserialize(de)?;
//         let ans = column1
//             .data
//             .into_iter()
//             .zip(column2.data.into_iter())
//             .map(|(id, name)| Self {
//                 id,
//                 name: name.into_owned(),
//             })
//             .collect();
//         Ok(ans)
//     }
// }

fn main() {
    println!("Hello, world!");
}
