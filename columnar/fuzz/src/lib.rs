use arbitrary::Arbitrary;
use serde_columnar::{columnar, iterable::*};
use std::borrow::Cow;
use std::collections::HashMap;

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Default, Arbitrary, PartialEq)]
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

#[columnar(vec, map, ser, de, iterable)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct VecStore<'a> {
    #[columnar(class = "vec", iter = "Data<'a>")]
    data: Vec<Data<'a>>,
    #[columnar(class = "vec")]
    data2: Vec<Data<'a>>,
    #[columnar(strategy = "DeltaRle")]
    id: u64,
    #[columnar(borrow)]
    borrow_str: Cow<'a, str>,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct MapStore<'a> {
    #[columnar(class = "map")]
    data: HashMap<u64, Data<'a>>,
    id: u64,
}

#[columnar(ser, de)]
#[derive(Debug, Clone, Arbitrary, PartialEq)]
pub struct NestedStore<'a> {
    #[columnar(class = "vec", iter = "VecStore<'a>")]
    stores: Vec<VecStore<'a>>,
    #[columnar(class = "map")]
    map_stores: HashMap<u64, VecStore<'a>>,
}
