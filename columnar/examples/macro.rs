use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_columnar::columnar;

#[columnar(map, vec, ser, de)]
#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
struct Data {
    id: u64,
    s: String,
}

// TODO: index checker

#[columnar(compatible, vec, ser, de)]
#[derive(Debug)]
struct Store {
    id: u64,
    s: String,
    #[columnar(class = "vec", optional, index = 0)]
    data: Vec<Data>,
    #[columnar(class = "map", optional, index = 1)]
    map: HashMap<String, Data>,
}

fn main() {}
