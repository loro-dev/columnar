use serde_columnar::{columnar, from_bytes, iter_from_bytes, iterable::*, to_vec};
use std::{borrow::Cow, collections::HashMap, vec};

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Default, PartialEq)]
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

#[test]
fn fuzz_vec() {
    let data = Data {
        rle: 255,
        delta_rle_uint: 612771024299098111,
        bool_rle: true,
        tuple_rle: (33023, "".to_string()),
        borrow_str: "".into(),
        skip: true,
        vec: vec![],
        map: Default::default(),
        optional: 0,
        borrow_optional_bytes: "".into(),
    };
    let buf = to_vec(&data).unwrap();
    println!("bytes {:?}", &buf);
    // let v = vec![];
    // let data = ::serde_columnar::ColumnarVec::<_, Vec<Data>>::new(&v);
    // let buf = to_vec(&data).unwrap();
    // println!("data {:?}", &buf);
    let _store: Data = from_bytes(&buf).unwrap();
    println!("data {:?}", _store);
    let _store2 = iter_from_bytes::<Data>(&buf).unwrap();
    // assert_eq!(store, store2);
}
