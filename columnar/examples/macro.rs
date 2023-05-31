use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_columnar::columnar;

#[columnar(map, vec, ser, de)]
#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
struct Data {
    id: u64,
    s: String,
}

#[columnar(compatible, vec, ser, de)]
#[derive(Debug)]
struct Store {
    id: u64,
    s: String,
    #[columnar(class = "vec")]
    data: Vec<Data>,
    #[columnar(class = "map")]
    map: HashMap<String, Data>,
}


fn main() {}
