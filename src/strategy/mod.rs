mod delta;
mod rle;
pub(crate) use rle::{AnyRleDecoder, AnyRleEncoder, BoolRleEncoder, DeRle, Rle};
use serde::{Deserialize, Serialize};

use crate::ColumnarError;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Strategy {
    Rle = 1,
    BoolRle,
    Delta,
    DeltaRle,
}

impl TryFrom<u8> for Strategy {
    type Error = ColumnarError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Strategy::Rle),
            2 => Ok(Strategy::BoolRle),
            3 => Ok(Strategy::Delta),
            4 => Ok(Strategy::DeltaRle),
            _ => Err(ColumnarError::InvalidStrategy(value)),
        }
    }
}
