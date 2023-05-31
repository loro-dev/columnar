pub mod bool_rle;
pub mod delta_rle;
pub mod rle;
pub mod serde_impl;

use std::fmt::Debug;

#[cfg(feature = "compress")]
use crate::compress::CompressConfig;
use crate::Strategy;
use crate::{columnar_internal::ColumnarEncoder, ColumnarDecoder, ColumnarError};

pub(crate) trait ColumnTrait {
    const STRATEGY: Strategy;
    fn attr(&self) -> &ColumnAttr;
    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError>;
    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized;
}

/// The attributes of a column
///
/// including compress config and some ones that may be used in the future.
#[cfg(feature = "compress")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnAttr {
    pub index: usize,
    pub compress: Option<CompressConfig>,
}

#[cfg(not(feature = "compress"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnAttr {
    pub index: usize,
}

impl ColumnAttr {
    pub(crate) fn empty() -> Self {
        #[cfg(feature = "compress")]
        return Self {
            index: 0,
            compress: None,
        };
        #[cfg(not(feature = "compress"))]
        return Self { index: 0 };
    }
}

// TODO: remove this
pub struct ColumnEncoder {
    ser: ColumnarEncoder,
}

impl ColumnEncoder {
    pub(crate) fn new() -> Self {
        Self {
            ser: ColumnarEncoder::new(),
        }
    }

    #[inline]
    #[allow(unused_variables)]
    fn encode_strategy_index_compress<T: ColumnTrait>(
        encoded_buf: Vec<u8>,
        column: &T,
    ) -> Result<Vec<u8>, ColumnarError> {
        let mut ans = Vec::new();
        #[cfg(feature = "compress")]
        {
            let attr = column.attr();
            if let Some(ref config) = attr.compress {
                if encoded_buf.len() < config.threshold {
                    use flate2::bufread::DeflateEncoder;
                    use std::io::Read;
                    ans.push(1);
                    let mut encoder =
                        DeflateEncoder::new(encoded_buf.as_slice(), config.compression);
                    encoder.read_to_end(&mut ans)?;
                    return Ok(ans);
                }
            }
        }
        ans.push(0);
        ans.extend(encoded_buf);
        Ok(ans)
    }

    pub(crate) fn encode<T: ColumnTrait>(mut self, column: &T) -> Result<Vec<u8>, ColumnarError> {
        column.encode(&mut self.ser)?;
        let buf = self.ser.into_bytes();
        let ans = Self::encode_strategy_index_compress(buf, column)?;
        Ok(ans)
    }
}

// TODO: remove this
pub struct ColumnDecoder<'b> {
    original_bytes: &'b [u8],
}

impl<'b> ColumnDecoder<'b> {
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Self {
            original_bytes: bytes,
        }
    }

    pub(crate) fn decode<T: ColumnTrait>(&mut self) -> Result<T, ColumnarError> {
        let (compress, bytes) = self.original_bytes.split_at(1);
        #[cfg(feature = "compress")]
        {
            let compress = compress[0];
            let mut output = Vec::new();
            let bytes = if compress == 0 {
                bytes
            } else {
                use flate2::bufread::DeflateDecoder;
                use std::io::Read;
                let mut decoder = DeflateDecoder::new(bytes);
                decoder.read_to_end(&mut output)?;
                &output
            };
            let mut columnar_decoder = ColumnarDecoder::new(bytes);
            let column = T::decode(&mut columnar_decoder)?;
            return Ok(column);
        }
        #[cfg(not(feature = "compress"))]
        {
            if compress[0] != 0 {
                return Err(ColumnarError::ColumnarEncodeError(
                    "The `compress` feature is disable and try to decode compressed bytes".into(),
                ));
            }
            let mut columnar_decoder = ColumnarDecoder::new(bytes);
            let column = T::decode(&mut columnar_decoder)?;
            Ok(column)
        }
    }
}
