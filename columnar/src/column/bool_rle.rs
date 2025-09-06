use crate::{
    strategy::{BoolRleDecoder, BoolRleEncoder},
    ColumnAttr, ColumnarError,
};

use super::ColumnTrait;

/// The Column that is scheduled to be compressed using [BoolRleEncoder]
#[derive(Debug)]
pub struct BoolRleColumn {
    pub data: Vec<bool>,
    pub attr: ColumnAttr,
}

impl BoolRleColumn {
    pub fn new(data: Vec<bool>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl ColumnTrait for BoolRleColumn {
    fn attr(&self) -> ColumnAttr {
        self.attr
    }
    fn len(&self) -> usize {
        self.data.len()
    }

    fn encode(&self) -> Result<Vec<u8>, ColumnarError> {
        let mut rle_encoder = BoolRleEncoder::new();
        for data in self.data.iter() {
            rle_encoder.append(*data)?
        }
        rle_encoder.finish()
    }

    fn decode(bytes: &[u8]) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut bool_rle_decoder = BoolRleDecoder::new(bytes);
        Ok(Self {
            data: bool_rle_decoder.decode()?,
            attr: ColumnAttr::empty(),
        })
    }
}
