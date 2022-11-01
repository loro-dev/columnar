use std::any::TypeId;

use crate::{ColumnarDecoder, ColumnarEncoder, ColumnarError};

use super::{AnyRleDecoder, AnyRleEncoder};

pub(crate) struct DeltaRleEncoder<'a> {
    rle: AnyRleEncoder<'a, i64>,
    absolute_value: i64,
}

impl<'a> DeltaRleEncoder<'a> {
    pub(crate) fn new(ser: &'a mut ColumnarEncoder) -> Self {
        Self {
            rle: AnyRleEncoder::new(ser),
            absolute_value: 0,
        }
    }

    pub(crate) fn append(&mut self, value: i64) -> Result<(), ColumnarError> {
        let delta = value.saturating_sub(self.absolute_value);
        self.absolute_value = value;
        self.rle.append(&delta)
    }

    pub(crate) unsafe fn append_any<T>(&mut self, value: &T) -> Result<(), ColumnarError> {
        // let value: i64 = if TypeId::of::<T>() == TypeId::of::<u8>() {
        //     let t: u8 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<u16>() {
        //     let t: u16 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<u32>() {
        //     let t: u32 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<u64>() {
        //     let t: u64 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<i8>() {
        //     let t: i8 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<i16>() {
        //     let t: i16 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<i32>() {
        //     let t: i32 = std::mem::transmute_copy(value);
        //     t as i64
        // } else if TypeId::of::<T>() == TypeId::of::<i64>() {
        //     let t: i64 = std::mem::transmute_copy(value);
        //     t as i64
        // } else {
        //     return Err(ColumnarError::RleEncodeError(
        //         "only num type can be encoded by delta encoder".to_string(),
        //     ));
        // };
        let value: i64 = std::mem::transmute_copy(value);
        let delta = value.saturating_sub(self.absolute_value);
        self.absolute_value = value;
        self.rle.append(&delta)
    }

    pub(crate) fn finish(self) -> Result<(), ColumnarError> {
        self.rle.finish()
    }
}

pub(crate) struct DeltaRleDecoder<'a, 'de> {
    rle: AnyRleDecoder<'a, 'de, i64>,
    absolute_value: i64,
}

impl<'a, 'de> DeltaRleDecoder<'a, 'de> {
    pub(crate) fn new(de: &'a mut ColumnarDecoder<'de>) -> Self {
        Self {
            rle: AnyRleDecoder::new(de),
            absolute_value: 0,
        }
    }

    pub(crate) fn decode(&mut self) -> Result<Vec<i64>, ColumnarError> {
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

    fn try_next(&mut self) -> Result<Option<i64>, ColumnarError> {
        if let Some(delta) = self.rle.try_next()? {
            self.absolute_value = self.absolute_value.saturating_add(delta);
            Ok(Some(self.absolute_value))
        } else {
            Ok(None)
        }
    }
}