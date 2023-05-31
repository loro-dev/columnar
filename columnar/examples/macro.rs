use serde_columnar::columnar;

#[columnar(compatible, vec, ser, de)]
#[derive(Debug)]
struct Data {
    id: u64,
    s: String,
}

fn main() {}
