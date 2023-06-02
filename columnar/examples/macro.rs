use serde::{Deserialize, Serialize};
use serde_columnar::columnar;

#[columnar(ser, de)]
struct B<P>
where
    P: Serialize + for<'a> Deserialize<'a>,
{
    t: P,
}
fn main() {}
