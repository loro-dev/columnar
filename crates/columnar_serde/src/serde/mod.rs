use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde_with::SerializeAs;

use crate::{columnar::ColumnarEncoder, columnar_impl::Columnar, Columns, Row};

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
        let mut columnar = Columnar::new();
        let bytes = columnar.encode(&self).unwrap();
        serializer.serialize_bytes(bytes.as_slice())
    }
}
