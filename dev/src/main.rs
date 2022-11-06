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

fn main() {
    use std::borrow::Cow;
    println!("Hello, world!");

    let a = String::from("ad");
    let a = Cow::Borrowed(&a);
    println!("{:?}", a);
}
