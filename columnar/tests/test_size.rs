use columnar::columnar;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
lazy_static! {
    static ref STORE: VecStore = {
        let mut _data = Vec::new();
        for i in 0..10000 {
            _data.push(Data {
                id: i / 50,
                name: format!("name{}", i),
            });
        }
        VecStore { data: _data }
    };
    static ref NORMAL_STORE: NormalStore = {
        let mut _data = Vec::new();
        for i in 0..10000 {
            _data.push(NormalData {
                id: i / 50,
                name: format!("name{}", i),
            });
        }
        NormalStore { data: _data }
    };
}

#[columnar(vec)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u32,
    name: String,
}

#[columnar]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VecStore {
    #[columnar(type = "vec")]
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalData {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalStore {
    pub data: Vec<NormalData>,
}

#[test]
fn test_size() {
    // columnar
    let bytes = columnar::to_vec(&*STORE).unwrap();
    let columnar_size = bytes.len();
    let store = columnar::from_bytes::<VecStore>(&bytes).unwrap();
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