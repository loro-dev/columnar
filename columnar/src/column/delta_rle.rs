use crate::{
    strategy::{DeltaRleDecoder, DeltaRleEncoder},
    ColumnAttr, ColumnarDecoder, ColumnarEncoder, ColumnarError, Strategy,
};

use super::{rle::Rleable, ColumnTrait};

pub trait DeltaRleable: Rleable + Copy + TryFrom<i128> + TryInto<i128> {}

impl<T> DeltaRleable for T where T: Rleable + Copy + TryFrom<i128> + TryInto<i128> {}

/// The Column that is scheduled to be compressed using [DeltaRleEncoder]
#[derive(Debug)]
pub struct DeltaRleColumn<T> {
    pub data: Vec<T>,
    attr: ColumnAttr,
}

impl<T> DeltaRleColumn<T> {
    pub fn new(data: Vec<T>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl<T> ColumnTrait for DeltaRleColumn<T>
where
    T: DeltaRleable,
{
    const STRATEGY: Strategy = Strategy::DeltaRle;
    fn attr(&self) -> &ColumnAttr {
        &self.attr
    }
    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError> {
        let mut delta_rle = DeltaRleEncoder::new(columnar_encoder);
        for &data in self.data.iter() {
            delta_rle
                .append(data.try_into().map_err(|_e| {
                    ColumnarError::RleEncodeError("cannot into i128".to_string())
                })?)?
        }

        delta_rle.finish()
    }

    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut delta_rle_decoder = DeltaRleDecoder::new(columnar_decoder);
        let data = delta_rle_decoder.decode()?;
        Ok(Self {
            data,
            attr: ColumnAttr::empty(),
        })
    }
}
