use std::borrow::Cow;


use serde_columnar::columnar;

#[columnar(ser, de)]
#[derive(Debug, PartialEq)]
struct A<'a> {
    a: u64,
    #[columnar(borrow, optional, index = 0)]
    b: Cow<'a, str>,
}
fn main() {}
