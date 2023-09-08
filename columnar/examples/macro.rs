use std::collections::BTreeMap;

use serde_columnar::columnar;

#[columnar(vec, map, ser, de)]
#[derive(Debug, Clone, PartialEq)]
struct A {
    a: u64,
}

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
