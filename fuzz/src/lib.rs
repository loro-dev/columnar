use arbitrary::Arbitrary;
use serde_columnar::{columnar, iterable::*};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Default, Arbitrary, PartialEq)]
pub struct Data<'a> {
    #[columnar(strategy = "Rle")]
    pub rle: u8,
    #[columnar(strategy = "DeltaRle")]
    pub delta_rle_uint: u64,
    #[columnar(strategy = "BoolRle")]
    pub bool_rle: bool,
    #[columnar(strategy = "DeltaOfDelta")]
    pub delta_of_delta: i64,
    #[columnar(strategy = "Rle")]
    pub tuple_rle: (u16, String),
    #[columnar(borrow)]
    pub borrow_str: Cow<'a, str>,
    // #[columnar(skip)]
    // pub skip: bool,
    #[columnar(class = "vec", iter = "Data<'a>")]
    pub vec: Vec<Data<'a>>,
    #[columnar(class = "map")]
    pub map: HashMap<String, Data<'a>>,
    #[columnar(optional, index = 0)]
    pub optional: u32,
    #[columnar(borrow, optional, index = 1)]
    pub borrow_optional_bytes: Cow<'a, str>,
}

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct VecStore<'a> {
    #[columnar(class = "vec", iter = "Data<'a>")]
    pub data: Vec<Data<'a>>,
    #[columnar(class = "vec")]
    pub data2: Vec<Data<'a>>,
    #[columnar(strategy = "DeltaRle")]
    pub id: u64,
    #[columnar(borrow)]
    pub borrow_str: Cow<'a, str>,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct MapStore<'a> {
    #[columnar(class = "map")]
    pub data: HashMap<u64, Data<'a>>,
    pub id: u64,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct NestedStore<'a> {
    #[columnar(class = "vec", iter = "VecStore<'a>")]
    pub stores: Vec<VecStore<'a>>,
    #[columnar(class = "map")]
    pub map_stores: HashMap<u64, VecStore<'a>>,
}

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct DeltaOfDelta {
    #[columnar(strategy = "DeltaOfDelta")]
    pub a: i64,
}

#[columnar(ser, de)]
#[derive(Clone, Arbitrary, PartialEq)]
pub struct DeltaOfDeltaStore {
    #[columnar(class = "vec")]
    pub data: Vec<DeltaOfDelta>,
}
impl Debug for DeltaOfDeltaStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeltaOfDeltaStore{\n")?;
        f.write_str(&format!("\tdata: vec!{:?}\n", self.data))?;
        f.write_str("}\n")
    }
}
