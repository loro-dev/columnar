use crate::{
    strategy::{BoolRleDecoder, BoolRleEncoder},
    ColumnAttr, ColumnarDecoder, ColumnarEncoder, ColumnarError, Strategy,
};

use super::ColumnTrait;

#[derive(Debug)]
pub struct BoolRleColumn {
    pub data: Vec<bool>,
    attr: ColumnAttr,
}

impl BoolRleColumn {
    pub fn new(data: Vec<bool>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl ColumnTrait for BoolRleColumn {
    const STRATEGY: Strategy = Strategy::BoolRle;
    fn attr(&self) -> &ColumnAttr {
        &self.attr
    }
    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError> {
        let mut rle_encoder = BoolRleEncoder::new(columnar_encoder);
        for data in self.data.iter() {
            rle_encoder.append(*data)?
        }
        rle_encoder.finish()?;
        Ok(())
    }

    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut bool_rle_decoder = BoolRleDecoder::new(columnar_decoder);
        Ok(Self {
            data: bool_rle_decoder.decode()?,
            attr: ColumnAttr::empty(),
        })
    }
}
