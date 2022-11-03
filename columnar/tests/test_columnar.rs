use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, ops::DerefMut};

use columnar::{
    fuzz::sample::{Data, VecStore},
    Column, ColumnAttr, ColumnarDecoder, ColumnarEncoder, Strategy, VecRow,
};

#[test]
fn test() {
    let store = VecStore {
        data: vec![
            Data {
                id: 0,
                name: "".to_string(),
            },
        ],
        id: 0,
    };
    let mut encoder = ColumnarEncoder::new();
    store.serialize(encoder.deref_mut()).unwrap();
    let buf = encoder.into_bytes();
    println!("{:?}", &buf);
    let mut decoder = ColumnarDecoder::new(&buf);
    let store2 = VecStore::deserialize(decoder.deref_mut()).unwrap();
    assert_eq!(store, store2);
}
