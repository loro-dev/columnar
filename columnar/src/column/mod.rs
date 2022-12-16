pub mod bool_rle;
pub mod delta_rle;
pub mod rle;
pub mod serde_impl;

use std::fmt::Debug;
use std::io::Read;

use flate2::bufread::DeflateDecoder;
use flate2::bufread::DeflateEncoder;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnAttr {
    pub index: usize,
    pub compress: Option<CompressConfig>,
}

impl ColumnAttr {
    pub(crate) fn empty() -> Self {
        Self {
            index: 0,
            compress: None,
        }
    }
}

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
    fn encode_strategy_index_compress<T: ColumnTrait>(
        encoded_buf: Vec<u8>,
        column: &T,
    ) -> Result<Vec<u8>, ColumnarError> {
        // TODO: forward backward compatible (strategy, index)
        let mut ans = Vec::new();
        let attr = column.attr();
        if let Some(ref config) = attr.compress {
            if encoded_buf.len() < config.threshold {
                ans.push(1);
                let mut encoder = DeflateEncoder::new(encoded_buf.as_slice(), config.compression);
                encoder.read_to_end(&mut ans)?;
                return Ok(ans);
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

pub(crate) struct ColumnDecoder<'b> {
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
        let compress = compress[0];
        let mut output = Vec::new();
        let bytes = if compress == 0 {
            bytes
        } else {
            let mut decoder = DeflateDecoder::new(bytes);
            decoder.read_to_end(&mut output)?;
            &output
        };
        let mut columnar_decoder = ColumnarDecoder::new(bytes);
        let column = T::decode(&mut columnar_decoder)?;
        Ok(column)
    }
}
