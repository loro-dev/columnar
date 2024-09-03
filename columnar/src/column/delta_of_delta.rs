use crate::{
    strategy::{DeltaOfDeltaDecoder, DeltaOfDeltaEncoder},
    ColumnAttr, ColumnarError, DeltaRleable, Strategy,
};

use super::ColumnTrait;

/// The Column that is scheduled to be compressed using [DeltaRleEncoder]
#[derive(Debug)]
pub struct DeltaOfDeltaColumn<T> {
    pub data: Vec<T>,
    pub attr: ColumnAttr,
}

impl<T> DeltaOfDeltaColumn<T> {
    pub fn new(data: Vec<T>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl<T> ColumnTrait for DeltaOfDeltaColumn<T>
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
        let mut delta_of_delta_rle = DeltaOfDeltaEncoder::new();
        for &data in self.data.iter() {
            delta_of_delta_rle
                .append(data.try_into().map_err(|_e| {
                    ColumnarError::RleEncodeError("cannot into i128".to_string())
                })?)?
        }

        delta_of_delta_rle.finish()
    }

    fn decode(bytes: &[u8]) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut delta_of_delta_decoder = DeltaOfDeltaDecoder::new(bytes);
        let data = delta_of_delta_decoder.decode()?;
        Ok(Self {
            data,
            attr: ColumnAttr::empty(),
        })
    }
}
