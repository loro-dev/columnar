mod rle;
pub use rle::{
    AnyRleDecoder, AnyRleEncoder, BoolRleDecoder, BoolRleEncoder, DeltaOfDeltaDecoder,
    DeltaOfDeltaEncoder, DeltaRleDecoder, DeltaRleEncoder,
};

use crate::ColumnarError;

/// The enum of Strategy includes `Rle`/`BoolRle`/`DeltaRle`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Strategy {
    Rle = 1,
    BoolRle,
    DeltaRle,
    DeltaOfDelta,
    None,
}

impl TryFrom<u8> for Strategy {
    type Error = ColumnarError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Strategy::Rle),
            2 => Ok(Strategy::BoolRle),
            3 => Ok(Strategy::DeltaRle),
            4 => Ok(Strategy::DeltaOfDelta),
            _ => Err(ColumnarError::InvalidStrategy(value)),
        }
    }
}

pub const MAX_RLE_COUNT: usize = 1e9 as usize;
