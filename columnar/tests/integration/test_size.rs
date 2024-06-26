use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_columnar::columnar;
lazy_static! {
    static ref STORE: VecStore = {
        let mut _data = Vec::new();
        for i in 0..10000 {
            _data.push(Data {
                id: i / 50,
                name: format!("name{}", i),
            });
        }
        VecStore { data: _data, id: 0 }
    };
    static ref NORMAL_STORE: NormalStore = {
        let mut _data = Vec::new();
        for i in 0..10000 {
            _data.push(NormalData {
                id: i / 50,
                name: format!("name{}", i),
            });
        }
        NormalStore { data: _data, id: 0 }
    };
}

type ID = u64;

#[columnar(vec, ser, de)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    #[columnar(strategy = "DeltaRle")]
    id: ID,
    #[columnar(strategy = "Rle")]
    name: String,
}

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecStore {
    #[columnar(class = "vec")]
    pub data: Vec<Data>,
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalData {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalStore {
    pub data: Vec<NormalData>,
    pub id: u32,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedStore {
    #[columnar(class = "vec")]
    stores: Vec<VecStore>,
    #[columnar(class = "map")]
    map_stores: HashMap<u64, VecStore>,
}

#[test]
fn test_size() {
    // columnar
    let bytes = serde_columnar::to_vec(&*STORE).unwrap();
    let columnar_size = bytes.len();
    let store = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();
    assert_eq!(store, *STORE);

    // postcard
    let bytes = postcard::to_allocvec(&*NORMAL_STORE).unwrap();
    let postcard_size = bytes.len();
    let store = postcard::from_bytes::<NormalStore>(&bytes).unwrap();
    assert_eq!(store, *NORMAL_STORE);

    // bincode
    let bytes = bincode::serialize(&*NORMAL_STORE).unwrap();
    let bincode_size = bytes.len();
    let store = bincode::deserialize::<NormalStore>(&bytes).unwrap();
    assert_eq!(store, *NORMAL_STORE);
    println!(
        "columnar: {}, postcard: {}, bincode: {}",
        columnar_size, postcard_size, bincode_size
    );
}
