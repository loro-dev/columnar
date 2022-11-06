use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, collections::HashMap};

use columnar::{columnar, to_vec, Column, ColumnAttr, ColumnarVec, MapRow, Strategy, VecRow};

type DeltaType = u32;

#[columnar(vec)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Data {
    #[columnar(strategy = "Rle")]
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
