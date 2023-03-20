mod delta;
pub use delta::{DeltaRleDecoder, DeltaRleEncoder};
mod rle;
pub use rle::{AnyRleDecoder, AnyRleEncoder, BoolRleDecoder, BoolRleEncoder};

use crate::ColumnarError;

/// The enum of Strategy includes `Rle`/`BoolRle`/`DeltaRle`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Strategy {
    Rle = 1,
    BoolRle,
    DeltaRle,
}

impl TryFrom<u8> for Strategy {
    type Error = ColumnarError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Strategy::Rle),
            2 => Ok(Strategy::BoolRle),
            3 => Ok(Strategy::DeltaRle),
            _ => Err(ColumnarError::InvalidStrategy(value)),
        }
    }
}

const MAX_RLE_COUNT: usize = 1e7 as usize;
