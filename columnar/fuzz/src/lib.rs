use arbitrary::{Arbitrary, Unstructured};
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use serde_columnar::{columnar, iterable::*};
use std::borrow::Cow;
use std::collections::HashMap;

fn arbitrary_float(u: &mut Unstructured) -> arbitrary::Result<f64> {
    u.arbitrary::<f64>()
        .map(|f| if f.is_nan() { 0.0 } else { f })
}

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Arbitrary, Default)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u8,
    #[columnar(strategy = "DeltaRle")]
    id2: u64,
    #[columnar(strategy = "Rle")]
    id3: usize,
    #[columnar(strategy = "DeltaRle")]
    id4: i64,
    id5: i128,
    #[arbitrary(with=arbitrary_float)]
    id6: f64,
    id7: (u16, i32),
    #[columnar(strategy = "BoolRle")]
    b: bool,
    name: String,
    #[columnar(class = "vec", iter = "Data")]
    vec: Vec<Data>,
    #[columnar(class = "map")]
    map: HashMap<String, Data>,
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.id2 == other.id2
            && self.id3 == other.id3
            && self.id4 == other.id4
            && self.id5 == other.id5
            && (self.id6 == other.id6 || (self.id6.is_nan() && other.id6.is_nan()))
            && self.id7 == other.id7
            && self.b == other.b
            && self.name == other.name
            && self.vec == other.vec
            && self.map == other.map
    }
}

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct VecStore<'a> {
    #[columnar(class = "vec", iter = "Data")]
    data: Vec<Data>,
    #[columnar(class = "vec")]
    data2: Vec<Data>,
    #[columnar(strategy = "DeltaRle")]
    id: u64,
    #[columnar(borrow)]
    borrow_str: Cow<'a, str>,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct MapStore {
    #[columnar(class = "map")]
    data: HashMap<u64, Data>,
    id: u64,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct NestedStore<'a> {
    #[columnar(class = "vec", iter = "VecStore<'a>")]
    stores: Vec<VecStore<'a>>,
    #[columnar(class = "map")]
    map_stores: HashMap<u64, VecStore<'a>>,
}
