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
        self.rle.append(&delta)
    }

    /// #Safety:
    ///
    /// when T is u8, u16, u32, ,u64, usize, i8, i16, i32, i64, isize, `append_any` is safe
    pub(crate) unsafe fn append_any<T>(&mut self, value: &T) -> Result<(), ColumnarError> {
        let padding = std::mem::size_of::<i128>() / std::mem::size_of::<T>();
        let value = match padding {
            1 => std::mem::transmute_copy(value),
            2 => {
                let value: u64 = std::mem::transmute_copy(value);
                value as i128
            }
            4 => {
                let value: u32 = std::mem::transmute_copy(value);
                value as i128
            }
            8 => {
                let value: u16 = std::mem::transmute_copy(value);
                value as i128
            }
            16 => {
                let value: u8 = std::mem::transmute_copy(value);
                value as i128
            }
            _ => {
                return Err(ColumnarError::RleEncodeError(
                    "only 8 & 16 & 32 num type can be encoded by delta encoder".to_string(),
                ));
            }
        };
        let delta = value.saturating_sub(self.absolute_value);
        self.absolute_value = value;
        self.rle.append(&delta)
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
    pub(crate) fn decode(&mut self) -> Result<Vec<i128>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    pub(crate) unsafe fn decode_to_any<T>(&mut self) -> Result<Vec<T>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = &self.try_next()? {
            // let value: T = if TypeId::of::<T>() == TypeId::of::<u8>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<u16>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<u32>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<u64>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<i8>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<i16>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<i32>() {
            //     std::mem::transmute_copy(value)
            // } else if TypeId::of::<T>() == TypeId::of::<i64>() {
            //     std::mem::transmute_copy(value)
            // } else {
            //     return Err(ColumnarError::RleEncodeError(
            //         "only num type can be decoded by delta encoder".to_string(),
            //     ));
            // };
            let value = std::mem::transmute_copy(value);
            values.push(value);
        }
        Ok(values)
    }

    fn try_next(&mut self) -> Result<Option<i128>, ColumnarError> {
        if let Some(delta) = self.rle.try_next()? {
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
        encoder.append(81020993).unwrap();
        encoder.append(20000000).unwrap();
        encoder.append(3).unwrap();
        encoder.append(4).unwrap();
        encoder.append(5).unwrap();
        encoder.finish().unwrap();
        let buf = columnar.into_bytes();
        println!("{:?}", buf);
        let mut decoder = ColumnarDecoder::new(&buf);
        let mut delta_rle_decoder = DeltaRleDecoder::new(&mut decoder);
        let values: Vec<i64> = unsafe { delta_rle_decoder.decode_to_any().unwrap() };
        // let values: Vec<i64> = delta_rle_decoder.decode().unwrap();
        println!("{:?}", values);
        assert_eq!(values, vec![81020993, 20000000, 3, 4, 5]);
    }
}
