use crate::{
    strategy::{DeltaRleDecoder, DeltaRleEncoder},
    ColumnAttr, ColumnarError, Strategy,
};

use super::{rle::Rleable, ColumnTrait};

pub trait DeltaRleable: Rleable + Copy + TryFrom<i128> + TryInto<i128> {}

impl<T> DeltaRleable for T where T: Rleable + Copy + TryFrom<i128> + TryInto<i128> {}

/// The Column that is scheduled to be compressed using [DeltaRleEncoder]
#[derive(Debug)]
pub struct DeltaRleColumn<T> {
    pub data: Vec<T>,
    pub attr: ColumnAttr,
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

    fn attr(&self) -> ColumnAttr {
        self.attr
    }
    fn len(&self) -> usize {
        self.data.len()
    }

    fn encode(&self) -> Result<Vec<u8>, ColumnarError> {
        let mut delta_rle = DeltaRleEncoder::new();
        for &data in self.data.iter() {
            delta_rle
                .append(data.try_into().map_err(|_e| {
                    ColumnarError::RleEncodeError("cannot into i128".to_string())
                })?)?
        }

        delta_rle.finish()
    }

    fn decode(bytes: &[u8]) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut delta_rle_decoder = DeltaRleDecoder::new(bytes);
        let data = delta_rle_decoder.decode()?;
        Ok(Self {
            data,
            attr: ColumnAttr::empty(),
        })
    }
}
