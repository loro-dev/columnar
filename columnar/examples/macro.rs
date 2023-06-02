use serde_columnar::columnar;
use std::collections::HashMap;

#[columnar(map, ser, de)]
#[derive(Debug, PartialEq, Clone)]
struct Data {
    id: u64,
    s: String,
    #[columnar(strategy = "Rle", optional, index = 0)]
    name: String,
}

#[columnar(vec, ser, de)]
#[derive(Debug)]
struct Store {
    id: u64,
    s: String,
    // #[columnar(class = "vec", optional, index = 0)]
    // data: Vec<Data>,
    #[columnar(class = "map", optional, index = 1)]
    map: HashMap<String, Data>,
}

fn main() {}
