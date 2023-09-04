use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_columnar::columnar;

#[columnar(ser, de)]
struct A<'a> {
    #[columnar(borrow)]
    data: Cow<'a, str>,
}

fn main() {}
