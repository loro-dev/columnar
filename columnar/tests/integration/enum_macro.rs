use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_columnar::{columnar, from_bytes, to_vec};
#[columnar(vec, map, ser, de)]
#[derive(PartialEq, Debug, Clone)]
struct Data {
    #[columnar(strategy = "DeltaRle")]
    id: u64,
    s: String,
}

#[columnar(ser, de)]
#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Enum {
    #[columnar(class = "vec")]
    Vec(Vec<Data>),
    #[columnar(class = "map")]
    Map(BTreeMap<u64, Data>),
}

#[test]
pub fn test_enum_macro_vec() {
    let mut vec = Vec::new();
    for i in 0..100 {
        vec.push(Data {
            id: i,
            s: "a".to_string(),
        });
    }
    let store = Enum::Vec(vec);
    let buf = to_vec(&store).unwrap();
    // println!("buf {:?}", &buf);
    // println!("buf len: {:?}", buf.len());
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}

#[test]
pub fn test_enum_macro_map() {
    let mut vec = Vec::new();
    for i in 0..100 {
        vec.push((
            i,
            Data {
                id: i,
                s: "a".to_string(),
            },
        ));
    }
    let store = Enum::Map(vec.into_iter().collect());
    let buf = to_vec(&store).unwrap();
    // println!("buf len: {:?}", buf.len());
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}
