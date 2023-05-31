use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_columnar::{columnar, from_bytes, to_vec};
#[columnar(vec, map, ser, de)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
struct Data {
    id: u64,
    id2: u64,
    s: String,
    s2: u8,
    #[columnar(skip)]
    s3: u8,
    #[columnar(skip)]
    st: String,
}

#[columnar(ser, de)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Store {
    #[columnar(class = "vec")]
    vec: Vec<Data>,
    #[columnar(class = "map")]
    map: HashMap<u64, Data>,
}

#[test]
pub fn test() {
    let store = Store {
        vec: vec![
            Data {
                id: 1,
                id2: 2,
                s: "a".to_string(),
                s2: 3,
                s3: 0,
                st: "".to_string(),
            },
            Data {
                id: 2,
                id2: 3,
                s: "b".to_string(),
                s2: 4,
                s3: 0,
                st: "".to_string(),
            },
        ],
        map: vec![
            (
                1,
                Data {
                    id: 1,
                    id2: 2,
                    s: "a".to_string(),
                    s2: 3,
                    s3: 0,
                    st: "".to_string(),
                },
            ),
            (
                2,
                Data {
                    id: 2,
                    id2: 3,
                    s: "b".to_string(),
                    s2: 4,
                    s3: 0,
                    st: "".to_string(),
                },
            ),
        ]
        .into_iter()
        .collect(),
    };
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}
