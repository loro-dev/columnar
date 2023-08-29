pub mod bool_rle;
pub mod delta_rle;
pub mod rle;
pub mod serde_impl;

#[cfg(feature = "compress")]
use crate::compress::CompressConfig;
use crate::{columnar_internal::ColumnarEncoder, ColumnarDecoder, ColumnarError};
use crate::{BoolRleColumn, DeltaRleColumn, DeltaRleable, RleColumn, Rleable, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::DerefMut;

pub trait ColumnTrait {
    const STRATEGY: Strategy;
    fn attr(&self) -> ColumnAttr;
    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError>;
    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The attributes of a column
///
/// including compress config and some ones that may be used in the future.
#[cfg(feature = "compress")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnAttr {
    pub index: Option<usize>,
    pub compress: Option<CompressConfig>,
}

// TODO: remove index
#[cfg(not(feature = "compress"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnAttr {
    pub index: Option<usize>,
}

impl ColumnAttr {
    pub(crate) fn empty() -> Self {
        #[cfg(feature = "compress")]
        return Self {
            index: None,
            compress: None,
        };
        #[cfg(not(feature = "compress"))]
        return Self { index: None };
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

impl From<Vec<bool>> for BoolRleColumn {
    fn from(value: Vec<bool>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl<T: DeltaRleable> From<Vec<T>> for DeltaRleColumn<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl<T: Rleable> From<Vec<T>> for RleColumn<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

impl<T> From<Vec<T>> for GenericColumn<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            data: value,
            attr: ColumnAttr::empty(),
        }
    }
}

#[derive(Debug)]
pub struct GenericColumn<T> {
    pub data: Vec<T>,
    pub attr: ColumnAttr,
}
impl<T> GenericColumn<T> {
    pub fn new(data: Vec<T>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

impl<T> ColumnTrait for GenericColumn<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    const STRATEGY: Strategy = Strategy::None;

    fn attr(&self) -> ColumnAttr {
        ColumnAttr::empty()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn encode(&self, columnar_encoder: &mut ColumnarEncoder) -> Result<(), ColumnarError> {
        self.data.serialize(columnar_encoder.deref_mut())?;
        Ok(())
    }

    fn decode(columnar_decoder: &mut ColumnarDecoder) -> Result<Self, ColumnarError>
    where
        Self: Sized,
    {
        let data = Deserialize::deserialize(columnar_decoder.deref_mut())?;
        Ok(Self {
            data,
            attr: ColumnAttr::empty(),
        })
    }
}
