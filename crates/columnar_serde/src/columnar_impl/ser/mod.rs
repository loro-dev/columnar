use serde::{Serialize, Serializer};
use serde_with::SerializeAs;

use crate::{Columns, Row, columnar_impl::ColumnarEncoder};

impl<T> SerializeAs<Vec<T>> for Columns<'_>
where
    T: Row,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let columns = Columns::from(source);
        columns.serialize(serializer)
    }
}

impl Serialize for Columns<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut columnar = ColumnarEncoder::new();
        columnar.encode(&self).unwrap();
        let bytes = columnar.finish();
        serializer.serialize_bytes(bytes.as_slice())
    }
}
