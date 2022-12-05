use serde::{Deserialize, Serialize};

use crate::{
    strategy::{AnyRleDecoder, AnyRleEncoder},
    ColumnAttr, ColumnarDecoder, ColumnarEncoder, ColumnarError, Strategy,
};

use super::ColumnTrait;

pub trait Rleable: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> {}
impl<T> Rleable for T where T: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> {}

#[derive(Debug)]
pub struct RleColumn<T> {
    pub data: Vec<T>,
    attr: ColumnAttr,
}

impl<T: Rleable> RleColumn<T> {
    pub fn new(data: Vec<T>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl<T> ColumnTrait for RleColumn<T>
where
    T: Rleable,
{
    const STRATEGY: Strategy = Strategy::Rle;
    fn attr(&self) -> &ColumnAttr {
        &self.attr
    }
    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError> {
        let mut rle_encoder = AnyRleEncoder::<T>::new(columnar_encoder);
        for data in self.data.iter() {
            rle_encoder.append(data)?
        }
        rle_encoder.finish()?;
        Ok(())
    }

    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut rle_decoder = AnyRleDecoder::new(columnar_decoder);
        Ok(Self {
            data: rle_decoder.decode()?,
            attr: ColumnAttr::empty(),
        })
    }
}
