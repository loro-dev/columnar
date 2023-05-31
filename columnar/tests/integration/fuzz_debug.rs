use serde::{Deserialize, Serialize};
use serde_columnar::{columnar, from_bytes, to_vec};
use std::collections::HashMap;

type ID = u64;

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u8,
    #[columnar(strategy = "DeltaRle")]
    id2: ID,
    #[columnar(strategy = "Rle")]
    id3: usize,
    #[columnar(strategy = "DeltaRle")]
    id4: i64,
    id5: i128,
    id6: f64,
    id7: (u16, i32),
    #[columnar(strategy = "BoolRle")]
    b: bool,
    #[columnar(compress)]
    name: String,
    #[columnar(class = "vec")]
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

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VecStore {
    #[columnar(class = "vec")]
    data: Vec<Data>,
    #[columnar(strategy = "DeltaRle")]
    id: u64,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapStore {
    #[columnar(class = "map")]
    data: HashMap<u64, Data>,
    id: u64,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NestedStore {
    #[columnar(class = "vec")]
    stores: Vec<VecStore>,
    #[columnar(class = "map")]
    map_stores: HashMap<u64, VecStore>,
}

#[test]
fn fuzz_vec() {
    let store = VecStore {
        data: vec![Data {
            id: 10,
            id2: 72057300834811470,
            id3: 72058667729420288,
            id4: 256,
            id5: 253,
            id6: 0.0,
            id7: (0, 0),
            b: false,
            name: "".into(),
            vec: vec![],
            map: HashMap::new(),
        }],
        id: 0,
    };
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}
