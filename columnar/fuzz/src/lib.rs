use arbitrary::{Arbitrary, Unstructured};
use columnar::columnar;
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use std::collections::HashMap;

type ID = u64;

fn arbitrary_float(u: &mut Unstructured) -> arbitrary::Result<f64> {
    u.arbitrary::<f64>()
        .map(|f| if f.is_nan() { 0.0 } else { f })
}

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Default)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u8,
    #[columnar(strategy = "DeltaRle", original_type = "u64")]
    id2: ID,
    #[columnar(strategy = "Rle")]
    id3: usize,
    #[columnar(strategy = "DeltaRle", original_type = "i64")]
    id4: i64,
    id5: i128,
    #[arbitrary(with=arbitrary_float)]
    id6: f64,
    id7: (u16, i32),
    #[columnar(strategy = "BoolRle")]
    b: bool,
    #[columnar(compress)]
    name: String,
    #[columnar(type = "vec")]
    vec: Vec<Data>,
    #[columnar(type = "map")]
    map: HashMap<String, Data>,
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
