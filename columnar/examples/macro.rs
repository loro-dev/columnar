use serde::{Deserialize, Serialize};
use serde_columnar::columnar;

#[columnar(vec, ser, de)]
struct B<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Clone,
{
    t: P,
}

fn main() {}
