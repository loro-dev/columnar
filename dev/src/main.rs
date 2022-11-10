use columnar::columnar;
use serde::{Deserialize, Serialize};

#[columnar(map, de)]
#[derive(Serialize, Deserialize)]
struct Data {
    id: u64,
}

// #[columnar]
// struct Store {
//     #[columnar(type = "vec")]
//     data: Vec<Data>,
// }

fn main() {
    println!("Hello, world!");
}
