use std::marker::PhantomData;

use crate::{ColumnarDecoder, ColumnarEncoder, ColumnarError, DeltaRleable};

use super::{AnyRleDecoder, AnyRleEncoder};

pub struct DeltaRleEncoder<'a> {
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

pub struct DeltaRleDecoder<'a, 'de, T> {
    rle: AnyRleDecoder<'a, 'de, i128>,
    absolute_value: i128,
    _t: PhantomData<T>,
}

impl<'a, 'de, T: DeltaRleable> DeltaRleDecoder<'a, 'de, T> {
    pub(crate) fn new(de: &'a mut ColumnarDecoder<'de>) -> Self {
        Self {
            rle: AnyRleDecoder::new(de),
            absolute_value: 0,
            _t: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn decode(&mut self) -> Result<Vec<T>, ColumnarError> {
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

    pub fn try_next(&mut self) -> Result<Option<i128>, ColumnarError> {
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
        encoder.append(1).unwrap();
        encoder.append(2).unwrap();
        encoder.append(3).unwrap();
        encoder.append(4).unwrap();
        encoder.append(5).unwrap();
        encoder.append(6).unwrap();
        encoder.finish().unwrap();
        let buf = columnar.into_bytes();
        println!("{:?}", buf);
        let mut decoder = ColumnarDecoder::new(&buf);
        let mut delta_rle_decoder = DeltaRleDecoder::new(&mut decoder);
        let values: Vec<u64> = delta_rle_decoder.decode().unwrap();
        assert_eq!(values, vec![1, 2, 3, 4, 5, 6]);
    }
}
