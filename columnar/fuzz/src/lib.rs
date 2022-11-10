use arbitrary::Arbitrary;
use columnar::columnar;
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use std::collections::HashMap;

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u32,
    name: String,
}

#[columnar(vec, map, ser, de)]
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
    #[columnar(type = "map")]
    map_stores: HashMap<u64, VecStore>,
}
