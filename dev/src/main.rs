use columnar::columnar;
use serde::{Deserialize, Serialize};

#[columnar(map, ser, de)]
#[derive(Serialize, Deserialize)]
struct Data {
    id: u64,
    id2: u64,
    s: String,
}

// #[columnar]
// struct Store {
//     #[columnar(type = "vec")]
//     data: Vec<Data>,
// }

fn main() {
    println!("Hello, world!");
}
