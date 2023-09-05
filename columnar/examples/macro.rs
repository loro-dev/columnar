use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_columnar::columnar;

#[columnar(ser, de)]
struct A<'a> {
    #[columnar(borrow)]
    data: Cow<'a, str>,
    default: u64,
    #[columnar(optional, index = 0)]
    a: u32,
}

fn main() {}
