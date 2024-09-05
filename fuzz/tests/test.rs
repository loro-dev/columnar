use std::{borrow::Cow, collections::HashMap};

use columnar_fuzz::{Data, DeltaOfDelta, DeltaOfDeltaStore, NestedStore, VecStore};
use serde_columnar::{from_bytes, to_vec};

#[test]
fn test_delta_of_delta() {
    let store = NestedStore {
        stores: vec![VecStore {
            data: vec![Data {
                rle: 0,
                delta_rle_uint: 18446640719616540672,
                bool_rle: true,
                delta_of_delta: 2048,
                tuple_rle: (0, String::from("")),
                borrow_str: Cow::Borrowed(""),
                vec: vec![],
                map: HashMap::new(),
                optional: 0,
                borrow_optional_bytes: Cow::Borrowed(""),
            }],
            data2: vec![],
            id: 0,
            borrow_str: Cow::Borrowed(""),
        }],
        map_stores: HashMap::new(),
    };
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}

#[test]
fn test_dod_2() {
    let store = NestedStore {
        stores: vec![VecStore {
            data: vec![Data {
                rle: 117,
                delta_rle_uint: 905969664,
                bool_rle: true,
                delta_of_delta: 1097743111830,
                tuple_rle: (0, String::from("")),
                borrow_str: Cow::Borrowed(""),
                vec: vec![],
                map: HashMap::new(),
                optional: 0,
                borrow_optional_bytes: Cow::Borrowed(""),
            }],
            data2: vec![],
            id: 0,
            borrow_str: Cow::Borrowed(""),
        }],
        map_stores: HashMap::new(),
    };
    // 00001001 01101001 01101001 01101001 01100000
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}

#[test]
fn test_dod_3() {
    let store = VecStore {
        data: vec![Data {
            rle: 93,
            delta_rle_uint: 2449958197289549824,
            bool_rle: false,
            delta_of_delta: 127543348830722,
            tuple_rle: (0, "".to_string()),
            borrow_str: Cow::Borrowed("\u{1}\0\u{2}\"\"\0\0"),
            vec: vec![
                Data {
                    rle: 255,
                    delta_rle_uint: 18446744073709551615,
                    bool_rle: true,
                    delta_of_delta: -1,
                    tuple_rle: (6958, "1!!!!!!!!!!!!".to_string()),
                    borrow_str: Cow::Borrowed(""),
                    vec: vec![],
                    map: HashMap::new(),
                    optional: 9612,
                    borrow_optional_bytes: Cow::Borrowed("\0\0"),
                },
                Data {
                    rle: 0,
                    delta_rle_uint: 18012701965398346936,
                    bool_rle: false,
                    delta_of_delta: 2143231,
                    tuple_rle: (0, "".to_string()),
                    borrow_str: Cow::Borrowed(""),
                    vec: vec![],
                    map: HashMap::new(),
                    optional: 0,
                    borrow_optional_bytes: Cow::Borrowed(""),
                },
            ],
            map: HashMap::new(),
            optional: 0,
            borrow_optional_bytes: Cow::Borrowed(""),
        }],
        data2: vec![],
        id: 0,
        borrow_str: Cow::Borrowed(""),
    };
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}

#[test]
fn test_dod_4() {
    let store = DeltaOfDeltaStore {
        data: vec![
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
            DeltaOfDelta { a: -1 },
        ],
    };
    let buf = to_vec(&store).unwrap();
    println!("buf {:?}", &buf);
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
}
