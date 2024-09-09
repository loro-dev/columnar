/// Reference automerge implementation:
/// https://github.com/automerge/automerge-rs/blob/d7d2916acb17d23d02ae249763aa0cf2f293d880/rust/automerge/src/columnar/encoding/rle.rs
use crate::{
    column::{delta_of_delta::DeltaOfDeltable, rle::Rleable},
    columnar_internal::{ColumnarDecoder, ColumnarEncoder},
    ColumnarError, DeltaRleable,
};
use serde::{Deserialize, Serialize};

use std::{borrow::Borrow, marker::PhantomData, ops::DerefMut};

use super::MAX_RLE_COUNT;

#[derive(Default)]
pub struct BoolRleEncoder {
    ser: ColumnarEncoder,
    last: bool,
    count: usize,
}

impl BoolRleEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, value: bool) -> Result<(), ColumnarError> {
        if value == self.last {
            self.count += 1;
        } else {
            self.count.serialize(self.ser.deref_mut())?;
            self.last = value;
            self.count = 1;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, ColumnarError> {
        if self.count > 0 {
            self.count.serialize(self.ser.deref_mut()).unwrap();
        }
        Ok(self.ser.into_bytes())
    }
}

pub struct AnyRleEncoder<T> {
    ser: ColumnarEncoder,
    state: RleState<T>,
}

impl<T> AnyRleEncoder<T>
where
    T: Rleable,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<BT: Borrow<T>>(&mut self, value: BT) -> Result<(), ColumnarError> {
        self.append_value(value)
    }

    pub fn finish(mut self) -> Result<Vec<u8>, ColumnarError> {
        match self.take_state() {
            RleState::LoneVal(value) => self.flush_lit_run(vec![value]),
            RleState::Run(value, len) => self.flush_run(&value, len),
            RleState::LiteralRun(last, mut run) => {
                run.push(last);
                self.flush_lit_run(run);
            }
            RleState::Empty => {}
        };
        Ok(self.ser.into_bytes())
    }

    fn append_value<BT: Borrow<T>>(&mut self, value: BT) -> Result<(), ColumnarError> {
        self.state = match self.take_state() {
            RleState::Empty => RleState::LoneVal(value.borrow().clone()),
            RleState::LoneVal(other) => {
                if &other == value.borrow() {
                    RleState::Run(value.borrow().clone(), 2)
                } else {
                    let mut v = Vec::with_capacity(2);
                    v.push(other);
                    RleState::LiteralRun(value.borrow().clone(), v)
                }
            }
            RleState::Run(other, len) => {
                if &other == value.borrow() {
                    RleState::Run(other, len + 1)
                } else {
                    self.flush_run(&other, len);
                    RleState::LoneVal(value.borrow().clone())
                }
            }
            RleState::LiteralRun(last, mut run) => {
                if &last == value.borrow() {
                    self.flush_lit_run(run);
                    RleState::Run(value.borrow().clone(), 2)
                } else {
                    run.push(last);
                    RleState::LiteralRun(value.borrow().clone(), run)
                }
            }
        };
        Ok(())
    }

    fn take_state(&mut self) -> RleState<T> {
        let mut state = RleState::Empty;
        std::mem::swap(&mut self.state, &mut state);
        state
    }

    fn flush_run(&mut self, val: &T, len: usize) {
        self.encode_length(len as isize);
        self.encode_content(val.clone());
    }

    fn flush_lit_run(&mut self, run: Vec<T>) {
        self.encode_length(-(run.len() as isize));
        for val in run {
            self.encode_content(val);
        }
    }

    fn encode_content(&mut self, val: T) {
        val.serialize(self.ser.deref_mut()).unwrap();
    }

    fn encode_length(&mut self, val: isize) {
        val.serialize(self.ser.deref_mut()).unwrap();
    }
}

impl<T> Default for AnyRleEncoder<T> {
    fn default() -> Self {
        Self {
            ser: Default::default(),
            state: RleState::Empty,
        }
    }
}

#[derive(Debug)]
enum RleState<T> {
    Empty,
    LiteralRun(T, Vec<T>),
    LoneVal(T),
    Run(T, usize),
}

pub struct AnyRleDecoder<'de, T> {
    de: ColumnarDecoder<'de>,
    last_value: Option<T>,
    count: isize,
    literal: bool,
}

impl<'de, T> AnyRleDecoder<'de, T>
where
    T: Rleable,
{
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: ColumnarDecoder::new(bytes),
            last_value: None,
            count: 0,
            literal: false,
        }
    }

    pub fn decode(&mut self) -> Result<Vec<T>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    pub fn try_next(&mut self) -> Result<Option<T>, ColumnarError> {
        while self.count == 0 {
            let count = isize::deserialize(self.de.deref_mut());
            if count.is_err() {
                return Ok(None);
            }
            let count = count.unwrap();
            // Prevent bad data from causing oom loops
            if count.unsigned_abs() > MAX_RLE_COUNT {
                return Err(ColumnarError::RleDecodeError(format!(
                    "decode Rle count is too large : {}",
                    self.count
                )));
            }
            match count {
                n if n > 0 => {
                    self.count = n;
                    self.last_value = Some(T::deserialize(self.de.deref_mut())?);
                    self.literal = false;
                }
                n if n < 0 => {
                    self.count = n.abs();
                    self.literal = true;
                }
                _ => return Err(ColumnarError::RleDecodeError("Invalid count".to_string())),
            }
        }
        self.count -= 1;
        if self.literal {
            Ok(Some(T::deserialize(self.de.deref_mut())?))
        } else {
            Ok(self.last_value.clone())
        }
    }

    pub fn finalize(self) -> Result<&'de [u8], ColumnarError> {
        self.de.finalize()
    }
}

pub struct BoolRleDecoder<'de> {
    de: ColumnarDecoder<'de>,
    last_value: bool,
    count: usize,
}

impl<'de> BoolRleDecoder<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: ColumnarDecoder::new(bytes),
            last_value: true,
            count: 0,
        }
    }

    pub fn decode(&mut self) -> Result<Vec<bool>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    pub fn try_next(&mut self) -> Result<Option<bool>, ColumnarError> {
        while self.count == 0 {
            let count = usize::deserialize(self.de.deref_mut());
            if count.is_err() && self.count == 0 {
                return Ok(None);
            }
            self.count = count.unwrap();
            // Prevent bad data from causing oom loops
            if self.count > MAX_RLE_COUNT {
                return Err(ColumnarError::RleDecodeError(format!(
                    "decode Rle count is too large : {}",
                    self.count
                )));
            }
            self.last_value = !self.last_value;
        }
        self.count -= 1;
        Ok(Some(self.last_value))
    }

    pub fn finalize(self) -> Result<&'de [u8], ColumnarError> {
        self.de.finalize()
    }
}

#[derive(Default)]
pub struct DeltaRleEncoder {
    rle: AnyRleEncoder<i128>,
    absolute_value: i128,
}

impl DeltaRleEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<T: DeltaRleable>(&mut self, value: T) -> Result<(), ColumnarError> {
        let v: i128 = value
            .try_into()
            .map_err(|_| ColumnarError::RleEncodeError("cannot into i128".to_string()))?;
        let delta = v.saturating_sub(self.absolute_value);
        self.absolute_value = v;
        self.rle.append(delta)
    }

    pub fn finish(self) -> Result<Vec<u8>, ColumnarError> {
        self.rle.finish()
    }
}

pub struct DeltaRleDecoder<'de, T> {
    rle: AnyRleDecoder<'de, i128>,
    absolute_value: i128,
    _t: PhantomData<T>,
}

impl<'de, T: DeltaRleable> DeltaRleDecoder<'de, T> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            rle: AnyRleDecoder::new(bytes),
            absolute_value: 0,
            _t: PhantomData,
        }
    }

    pub fn decode(&mut self) -> Result<Vec<T>, ColumnarError> {
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

    pub fn finalize(self) -> Result<&'de [u8], ColumnarError> {
        self.rle.finalize()
    }
}

const MAX_DELTA_OF_DELTA: i64 = 1 << 20;

#[derive(Debug)]
pub struct DeltaOfDeltaEncoder {
    bits: Vec<u64>,
    last_used_bit: u8,
    head_num: Option<i64>,
    prev_value: i64,
    prev_delta: i64,
    use_bit: bool,
}

impl Default for DeltaOfDeltaEncoder {
    fn default() -> Self {
        Self {
            bits: vec![0u64],
            head_num: None,
            last_used_bit: 0,
            prev_value: 0,
            prev_delta: 0,
            use_bit: false,
        }
    }
}

impl DeltaOfDeltaEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, value: i64) -> Result<(), ColumnarError> {
        // println!("append value {}", value);
        if self.head_num.is_none() {
            self.head_num = Some(value);
            self.prev_value = value;
            return Ok(());
        }
        self.use_bit = true;
        let delta = value
            .checked_sub(self.prev_value)
            .ok_or(ColumnarError::RleEncodeError(
                "delta overflow 64 bits".to_string(),
            ))?;

        let delta_of_delta =
            delta
                .checked_sub(self.prev_delta)
                .ok_or(ColumnarError::RleEncodeError(
                    "delta of delta overflow 64 bits".to_string(),
                ))?;
        self.prev_value = value;
        self.prev_delta = delta;
        if delta_of_delta == 0 {
            self.write_bits(0, 1);
        } else if (-63..=64).contains(&delta_of_delta) {
            self.write_bits(0b10, 2);
            self.write_bits((delta_of_delta + 63) as u64, 7);
        } else if (-255..=256).contains(&delta_of_delta) {
            self.write_bits(0b110, 3);
            self.write_bits((delta_of_delta + 255) as u64, 9);
        } else if (-2047..=2048).contains(&delta_of_delta) {
            self.write_bits(0b1110, 4);
            self.write_bits((delta_of_delta + 2047) as u64, 12);
        } else if ((-MAX_DELTA_OF_DELTA + 1)..=MAX_DELTA_OF_DELTA).contains(&delta_of_delta) {
            self.write_bits(0b11110, 5);
            self.write_bits((delta_of_delta + MAX_DELTA_OF_DELTA - 1) as u64, 21);
        } else {
            self.write_bits(0b11111, 5);
            self.write_bits(delta_of_delta as u64, 64);
        }
        Ok(())
    }

    fn write_bits(&mut self, value: u64, count: u8) {
        if self.last_used_bit == 64 {
            self.bits.push(value << (64 - count));
            self.last_used_bit = count;
        } else {
            let remaining_bits = 64 - self.last_used_bit;
            if count > remaining_bits {
                let bits_latter = count - remaining_bits;
                let former = self.bits.last_mut().unwrap();
                *former ^= value >> bits_latter;
                self.bits.push(value << (64 - bits_latter));
                self.last_used_bit = bits_latter;
            } else {
                let last = self.bits.last_mut().unwrap();
                *last ^= value << (remaining_bits - count);
                self.last_used_bit += count;
            }
        }
    }

    #[inline(never)]
    pub fn finish(self) -> Result<Vec<u8>, ColumnarError> {
        let mut bytes = Vec::with_capacity(self.bits.len() * 8 + 1 + 8);
        if let Some(head_num) = self.head_num {
            bytes.extend_from_slice(&postcard::to_allocvec(&head_num)?);
        }
        let used = self.last_used_bit.div_ceil(8);
        bytes.push(if self.last_used_bit % 8 == 0 && self.use_bit {
            8
        } else {
            self.last_used_bit % 8
        });
        for bits in &self.bits[..self.bits.len() - 1] {
            bytes.extend(bits.to_be_bytes());
        }
        bytes.extend(&self.bits.last().unwrap().to_be_bytes()[..used as usize]);
        // println!("bits {:?}", &bytes);
        Ok(bytes)
    }
}

pub struct DeltaOfDeltaDecoder<'de, T> {
    bits: &'de [u8],
    head_num: Option<i64>,
    prev_value: i64,
    prev_delta: i64,
    index: usize,
    current_bits_index: u8,
    last_used_bit: u8,
    _t: PhantomData<T>,
}

impl<'de, T: DeltaOfDeltable> DeltaOfDeltaDecoder<'de, T> {
    pub fn new(bytes: &'de [u8]) -> Self {
        // println!("\ndecode bytes {:?}", &bytes);
        if bytes.len() < 2 {
            return Self {
                bits: bytes,
                head_num: None,
                prev_value: 0,
                prev_delta: 0,
                index: 0,
                current_bits_index: 0,
                last_used_bit: 0,
                _t: PhantomData,
            };
        }
        let (head_num, bytes) = postcard::take_from_bytes(bytes).unwrap();
        let last_used_bit = bytes[0];
        let bits = &bytes[1..];
        Self {
            bits,
            head_num: Some(head_num),
            prev_value: 0,
            prev_delta: 0,
            index: 0,
            current_bits_index: 0,
            last_used_bit,
            _t: PhantomData,
        }
    }

    pub fn decode(&mut self) -> Result<Vec<T>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<T>, ColumnarError> {
        if self.head_num.is_some() {
            self.prev_value = self.head_num.unwrap();
            self.head_num = None;
        } else {
            match self.read_bits(1) {
                Some(0) => self.prev_value += self.prev_delta,
                Some(1) => {
                    let (num_bits, bias) = if self.read_bits(1).unwrap() == 0 {
                        (7, 63)
                    } else if self.read_bits(1).unwrap() == 0 {
                        (9, 255)
                    } else if self.read_bits(1).unwrap() == 0 {
                        (12, 2047)
                    } else if self.read_bits(1).unwrap() == 0 {
                        (21, MAX_DELTA_OF_DELTA - 1)
                    } else {
                        (64, 0)
                    };
                    let delta_of_delta = self.read_bits(num_bits).unwrap() as i64 - bias;
                    self.prev_delta += delta_of_delta;
                    self.prev_value += self.prev_delta;
                }
                None => return Ok(None),
                _ => panic!("delta of delta read flag should be 0 or 1"),
            };
        }
        // println!("prev_value {}", self.prev_value);
        Ok(Some(self.prev_value.try_into().map_err(|_| {
            ColumnarError::RleDecodeError(format!(
                "{} cannot be safely converted from i128",
                self.prev_value
            ))
        })?))
    }

    fn read_bits(&mut self, count: u8) -> Option<u64> {
        if self.index >= self.bits.len() {
            return None;
        }

        let total_bits = (self.bits.len() - 1) * 8 + self.last_used_bit as usize;
        let read_bits = self.index * 8 + self.current_bits_index as usize;
        let remaining_bits = total_bits - read_bits;

        if remaining_bits < count as usize {
            return None;
        }

        let current_byte_remaining = 8 - self.current_bits_index;
        let ans = if count <= current_byte_remaining {
            let current_index = self.index;
            self.current_bits_index += count;
            if self.current_bits_index == 8 {
                self.index += 1;
                self.current_bits_index = 0;
            }
            let mask = u8::MAX >> (8 - count);
            let current_byte = self.bits[current_index];
            let after_shift = current_byte >> (current_byte_remaining - count);
            let ans = after_shift & mask;
            ans as u64
        } else {
            let mut ans = (self.bits[self.index] & u8::MAX >> (8 - current_byte_remaining)) as u64;
            self.index += 1;
            self.current_bits_index = 0;
            // read current_byte_remaining

            let mut rest = count - current_byte_remaining;
            // while per 8 bits
            while rest > 8 {
                // read 8 bits
                ans = (ans << 8) | self.bits[self.index] as u64;
                self.index += 1;
                rest -= 8;
            }
            // read rest bits
            ans = (ans << rest) | (self.bits[self.index] >> (8 - rest)) as u64;
            self.current_bits_index += rest;
            if self.current_bits_index == 8 {
                self.index += 1;
                self.current_bits_index = 0;
            }
            ans
        };
        Some(ans)
    }
}

impl<'de, T: Rleable> Iterator for AnyRleDecoder<'de, T> {
    type Item = Result<T, ColumnarError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl Iterator for BoolRleDecoder<'_> {
    type Item = Result<bool, ColumnarError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'de, T: DeltaRleable> Iterator for DeltaRleDecoder<'de, T> {
    type Item = Result<T, ColumnarError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(value)) => Some(T::try_from(value).map_err(|_| {
                ColumnarError::RleDecodeError(format!(
                    "{} cannot be safely converted from i128",
                    value
                ))
            })),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'de, T: DeltaOfDeltable> Iterator for DeltaOfDeltaDecoder<'de, T> {
    type Item = Result<T, ColumnarError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

mod test {

    #[test]
    fn test_rle() {
        use super::*;
        let mut rle_encoder = AnyRleEncoder::<u64>::new();
        rle_encoder.append(1000).unwrap();
        rle_encoder.append(1000).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.append(2).unwrap();
        let buf = rle_encoder.finish().unwrap();
        // println!("buf {:?}", &buf);
        let mut rle_decoder = AnyRleDecoder::<u64>::new(&buf);
        assert_eq!(rle_decoder.decode().unwrap(), vec![1000, 1000, 2, 2, 2]);
    }

    #[test]
    fn test_bool_rle() {
        use super::*;
        let mut rle_encoder = BoolRleEncoder::new();
        rle_encoder.append(true).unwrap();
        rle_encoder.append(true).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.append(false).unwrap();
        let buf = rle_encoder.finish().unwrap();
        assert_eq!(&buf, &[0, 2, 3]);
        let mut rle_decoder = BoolRleDecoder::new(&buf);
        assert_eq!(
            rle_decoder.decode().unwrap(),
            vec![true, true, false, false, false]
        );
    }

    #[test]
    fn test_delta_rle() {
        use super::*;
        let mut encoder = DeltaRleEncoder::new();
        encoder.append(1).unwrap();
        encoder.append(2).unwrap();
        encoder.append(3).unwrap();
        encoder.append(4).unwrap();
        encoder.append(5).unwrap();
        encoder.append(6).unwrap();
        let buf = encoder.finish().unwrap();
        // println!("{:?}", buf);
        let mut delta_rle_decoder = DeltaRleDecoder::new(&buf);
        let values: Vec<u64> = delta_rle_decoder.decode().unwrap();
        assert_eq!(values, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_delta_of_delta_rle() {
        use super::*;
        let mut encoder = DeltaOfDeltaEncoder::new();
        encoder.append(1).unwrap();
        encoder.append(2).unwrap();
        encoder.append(3).unwrap();
        encoder.append(4).unwrap();
        encoder.append(5).unwrap();
        encoder.append(6).unwrap();
        let buf = encoder.finish().unwrap();
        // println!("{:?}", buf);
        let mut delta_of_delta_rle_decoder = DeltaOfDeltaDecoder::new(&buf);
        let values: Vec<i64> = delta_of_delta_rle_decoder.decode().unwrap();
        assert_eq!(values, vec![1, 2, 3, 4, 5, 6]);
    }
}
