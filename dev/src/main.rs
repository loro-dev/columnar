use std::collections::HashMap;

use columnar::columnar;
use serde::{Deserialize, Serialize};
#[columnar(vec, map, ser, de)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Data {
    id: u64,
    id2: u64,
    s: String,
    #[columnar(compress)]
    s2: u8,
    #[columnar(compress(method = "fast"))]
    s3: u8,
    #[columnar(compress(min_size = 0, level = 9))]
    st: String,
}

#[columnar(ser, de)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum EnumData {
    #[columnar(type = "vec")]
    Data(Vec<Data>),
    #[columnar(type = "map")]
    Map(HashMap<u64, Data>),
}

fn main() {
    println!("Hello, world!");
    let store = EnumData::Data(vec![
        Data {
            id: 1,
            id2: 2,
            s: "s".to_string(),
            s2: 3,
            s3: 4,
            st: "1111111111111111".to_string(),
        },
        Data {
            id: 1,
            id2: 2,
            s: "s".to_string(),
            s2: 3,
            s3: 4,
            st: "11111111111111111".to_string(),
        },
        Data {
            id: 1,
            id2: 2,
            s: "s".to_string(),
            s2: 3,
            s3: 4,
            st: "11111111111111111".to_string(),
        },
    ]);
    let buf = columnar::to_vec(&store).unwrap();
    println!("buf len: {:?}", buf.len());
    let store2 = columnar::from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}
