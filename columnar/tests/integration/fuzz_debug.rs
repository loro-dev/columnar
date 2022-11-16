use columnar::{columnar, from_bytes, to_vec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type ID = u64;

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VecStore {
    #[columnar(type = "vec")]
    data: Vec<Data>,
    #[columnar(strategy = "DeltaRle")]
    id: u64,
}

#[columnar]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapStore {
    #[columnar(type = "map")]
    data: HashMap<u64, Data>,
    id: u64,
}

#[columnar]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NestedStore {
    #[columnar(type = "vec")]
    stores: Vec<VecStore>,
    #[columnar(type = "map")]
    map_stores: HashMap<u64, VecStore>,
}

#[test]
fn fuzz() {
    let data = vec![
        254, 10, 28, 0, 0, 0, 40, 40, 40, 40, 40, 40, 40, 40, 0, 0, 0, 0, 0, 0, 40,
    ];
    if let Ok(data) = from_bytes::<Data>(&data) {
        println!("########data: {:?}", data);
        let buf = to_vec(&data).unwrap();
        let data2 = from_bytes(&buf).unwrap();
        assert_eq!(data, data2);
    } else {
        println!("################failed to deserialize");
    }
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
