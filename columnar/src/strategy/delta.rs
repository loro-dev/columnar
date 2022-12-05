use crate::{ColumnarDecoder, ColumnarEncoder, ColumnarError};

use super::{AnyRleDecoder, AnyRleEncoder};

pub(crate) struct DeltaRleEncoder<'a> {
    rle: AnyRleEncoder<'a, i128>,
    absolute_value: i128,
}

impl<'a> DeltaRleEncoder<'a> {
    pub(crate) fn new(ser: &'a mut ColumnarEncoder) -> Self {
        Self {
            rle: AnyRleEncoder::new(ser),
            absolute_value: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn append(&mut self, value: i128) -> Result<(), ColumnarError> {
        let delta = value.saturating_sub(self.absolute_value);
        self.absolute_value = value;
        self.rle.append(delta)
    }

    pub(crate) fn finish(self) -> Result<(), ColumnarError> {
        self.rle.finish()
    }
}

pub(crate) struct DeltaRleDecoder<'a, 'de> {
    rle: AnyRleDecoder<'a, 'de, i128>,
    absolute_value: i128,
}

impl<'a, 'de> DeltaRleDecoder<'a, 'de> {
    pub(crate) fn new(de: &'a mut ColumnarDecoder<'de>) -> Self {
        Self {
            rle: AnyRleDecoder::new(de),
            absolute_value: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn decode<T: TryFrom<i128>>(&mut self) -> Result<Vec<T>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value.try_into().map_err(|_| {
                ColumnarError::RleDecodeError(format!(
                    "{} cannot be safely converted from i128",
                    value
                ))
            })?);
        }
        Ok(values)
    }

    fn try_next(&mut self) -> Result<Option<i128>, ColumnarError> {
        let next = self.rle.try_next()?;
        if let Some(delta) = next {
            self.absolute_value = self.absolute_value.saturating_add(delta);
            Ok(Some(self.absolute_value))
        } else {
            Ok(None)
        }
    }
}

mod test {
    #[test]
    fn test_delta_rle() {
        use super::*;
        let mut columnar = ColumnarEncoder::new();
        let mut encoder = DeltaRleEncoder::new(&mut columnar);
        encoder.append(0).unwrap();
        encoder.append(1).unwrap();
        encoder.finish().unwrap();
        let buf = columnar.into_bytes();
        println!("{:?}", buf);
        let mut decoder = ColumnarDecoder::new(&buf);
        let mut delta_rle_decoder = DeltaRleDecoder::new(&mut decoder);
        let values: Vec<u64> = delta_rle_decoder.decode().unwrap();
        assert_eq!(values, vec![0, 1]);
    }
}
