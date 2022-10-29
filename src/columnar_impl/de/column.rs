use crate::Columns;

use super::columnar::ColumnarDeserializer;

pub struct ColumnDecoder<'de> {
    // ColumnDecoder 中的 serde
    de: ColumnarDeserializer<'de>,
}

impl<'de, 'a: 'de> ColumnDecoder<'de> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            de: ColumnarDeserializer::new(bytes),
        }
    }

    pub(crate) fn decode_columns(&mut self) -> Columns<'a> {
        todo!()
    }
}
