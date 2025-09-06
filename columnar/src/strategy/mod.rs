mod rle;
pub use rle::{
    AnyRleDecoder, AnyRleEncoder, BoolRleDecoder, BoolRleEncoder, DeltaOfDeltaDecoder,
    DeltaOfDeltaEncoder, DeltaRleDecoder, DeltaRleEncoder,
};

pub const MAX_RLE_COUNT: usize = 1e9 as usize;
