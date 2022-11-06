use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, collections::HashMap, ops::DerefMut};

use columnar::{
    from_bytes,
    fuzz::sample::{Data, MapStore, NestedStore, VecStore},
    to_vec, Column, ColumnAttr, ColumnarDecoder, ColumnarEncoder, ColumnarError, Strategy, VecRow,
};

#[test]
fn test() {
    let store = VecStore {
        data: vec![Data {
            id: 0,
            name: "".to_string(),
        }],
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

#[test]
fn test_map() {
    let map: HashMap<u64, Data> = HashMap::from([(
        0,
        Data {
            id: 0,
            name: "".to_string(),
        },
    )]);
    let store = MapStore { data: map };
    let buf = to_vec(&store).unwrap();
    println!("{:?}", &buf);
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}

#[test]
fn test_nested() {
    use columnar::fuzz::sample::NestedStore;
    let store = NestedStore {
        stores: vec![],
        stores2: HashMap::new(),
    };
    let mut encoder = ColumnarEncoder::new();
    store.serialize(encoder.deref_mut()).unwrap();
    let buf = encoder.into_bytes();
    println!("{:?}", &buf);
    let mut decoder = ColumnarDecoder::new(&buf);
    let store2 = NestedStore::deserialize(decoder.deref_mut()).unwrap();
    assert_eq!(store, store2);
}
