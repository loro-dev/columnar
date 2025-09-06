use std::collections::BTreeMap;

use serde_columnar::columnar;

#[allow(dead_code)]
#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, PartialEq)]
struct A {
    a: u64,
}

#[allow(dead_code)]
#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, PartialEq)]
struct B {
    #[columnar(class = "vec")]
    vec: Vec<A>,
    #[columnar(class = "map")]
    map: BTreeMap<u8, A>,
    #[columnar(strategy = "BoolRle")]
    b: bool,
}

fn main() {}
