use columnar::columnar;
use serde::{Deserialize, Serialize};
#[columnar(vec, ser, de)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
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

fn main() {
    println!("Hello, world!");
    let store = Data {
        id: 1,
        id2: 2,
        s: "hello".to_string(),
        s2: 3,
        s3: 0,
        st: "".into(),
    };
    let buf = columnar::to_vec(&store).unwrap();
    let store2 = columnar::from_bytes::<Data>(&buf).unwrap();
    assert_eq!(store, store2);
}
