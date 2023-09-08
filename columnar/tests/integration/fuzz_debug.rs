use serde_columnar::{columnar, iter_from_bytes, iterable::*, to_vec};
use std::{borrow::Cow, collections::HashMap, vec};

#[columnar(vec, map, ser, de, iterable)]
#[derive(Clone, Default, PartialEq)]
pub struct Data<'a> {
    #[columnar(strategy = "Rle")]
    rle: u8,
    #[columnar(strategy = "DeltaRle")]
    delta_rle_uint: u64,
    #[columnar(strategy = "BoolRle")]
    bool_rle: bool,
    #[columnar(strategy = "Rle")]
    tuple_rle: (u16, String),
    #[columnar(borrow)]
    borrow_str: Cow<'a, str>,
    #[columnar(skip)]
    skip: bool,
    #[columnar(class = "vec", iter = "Data<'a>")]
    vec: Vec<Data<'a>>,
    #[columnar(class = "map")]
    map: HashMap<String, Data<'a>>,
    #[columnar(optional, index = 0)]
    optional: u32,
    #[columnar(borrow, optional, index = 1)]
    borrow_optional_bytes: Cow<'a, str>,
}

// #[columnar(vec, map, ser, de)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct VecStore {
//     #[columnar(class = "vec", iter = "Data")]
//     data: Vec<Data>,
//     #[columnar(class = "vec")]
//     data4: Vec<Data>,
//     #[columnar(strategy = "DeltaRle")]
//     id: u64,
//     #[columnar(borrow)]
//     borrow_str: Cow<'a, str>,
// }

// #[columnar(ser, de)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct MapStore {
//     #[columnar(class = "map")]
//     data: HashMap<u64, Data>,
//     id: u64,
// }

// #[columnar(ser, de)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct NestedStore {
//     #[columnar(class = "vec")]
//     stores: Vec<VecStore>,
//     #[columnar(class = "map")]
//     map_stores: HashMap<u64, VecStore>,
// }

#[test]
fn fuzz_vec() {
    // let store = VecStore {
    //     data: vec![],
    //     data4: vec![],
    //     // id: 0,
    //     // borrow_str: Cow::Borrowed(""),
    // };
    // let buf = to_vec(&store).unwrap();
    // println!("bytes {:?}", &buf);
    // let v = vec![];
    // let data = ::serde_columnar::ColumnarVec::<_, Vec<Data>>::new(&v);
    // let buf = to_vec(&data).unwrap();
    // println!("data {:?}", &buf);

    // let _store2 = iter_from_bytes::<VecStore>(&buf).unwrap();
    // assert_eq!(store, store2);
}
