use serde::{Deserialize, Serialize};

use crate::{
    strategy::{AnyRleDecoder, AnyRleEncoder},
    ColumnAttr, ColumnarError,
};

use super::ColumnTrait;

pub trait Rleable: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> {}
impl<T> Rleable for T where T: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> {}

/// The Column that is scheduled to be compressed using [AnyRleEncoder]
#[derive(Debug)]
pub struct RleColumn<T> {
    pub data: Vec<T>,
    pub attr: ColumnAttr,
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
    fn len(&self) -> usize {
        self.data.len()
    }

    fn attr(&self) -> ColumnAttr {
        self.attr
    }

    fn encode(&self) -> Result<Vec<u8>, ColumnarError> {
        let mut rle_encoder = AnyRleEncoder::<T>::new();
        for data in self.data.iter() {
            rle_encoder.append(data)?
        }
        rle_encoder.finish()
    }

    fn decode(bytes: &[u8]) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let mut rle_decoder = AnyRleDecoder::new(bytes);
        Ok(Self {
            data: rle_decoder.decode()?,
            attr: ColumnAttr::empty(),
        })
    }
}
