use serde_columnar::{columnar, iter_from_bytes, iterable::*, to_vec};
use std::{collections::HashMap, vec};
#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Default)]
pub struct Data {
    #[columnar(strategy = "Rle")]
    id: u8,
    #[columnar(strategy = "DeltaRle")]
    id2: u64,
    // #[columnar(strategy = "Rle")]
    // id3: usize,
    // #[columnar(strategy = "DeltaRle")]
    // id4: i64,
    // id5: i128,
    // id6: f64,
    // id7: (u16, i32),
    // #[columnar(strategy = "BoolRle")]
    // b: bool,
    // name: String,
    // #[columnar(class = "vec", iter = "Data")]
    // vec: Vec<Data>,
    // #[columnar(class = "map")]
    // map: HashMap<String, Data>,
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.id2 == other.id2
        // && self.id3 == other.id3
        // && self.id4 == other.id4
        // && self.id5 == other.id5
        // && (self.id6 == other.id6 || (self.id6.is_nan() && other.id6.is_nan()))
        // && self.id7 == other.id7
        // && self.b == other.b
        // && self.name == other.name
        // && self.vec == other.vec
        // && self.map == other.map
    }
}

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, PartialEq)]
pub struct VecStore {
    #[columnar(class = "vec", iter = "Data")]
    data: Vec<Data>,
    #[columnar(class = "vec")]
    data4: Vec<Data>,
    // #[columnar(strategy = "DeltaRle")]
    // id: u64,
    // #[columnar(borrow)]
    // borrow_str: Cow<'a, str>,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, PartialEq)]
pub struct MapStore {
    #[columnar(class = "map")]
    data: HashMap<u64, Data>,
    id: u64,
}

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
    let store = VecStore {
        data: vec![],
        data4: vec![],
        // id: 0,
        // borrow_str: Cow::Borrowed(""),
    };
    let buf = to_vec(&store).unwrap();
    println!("bytes {:?}", &buf);
    let v = vec![];
    let data = ::serde_columnar::ColumnarVec::<_, Vec<Data>>::new(&v);
    let buf = to_vec(&data).unwrap();
    println!("data {:?}", &buf);

    let _store2 = iter_from_bytes::<VecStore>(&buf).unwrap();
    // assert_eq!(store, store2);
}
