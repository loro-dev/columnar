/// Reference automerge implementation:
/// https://github.com/automerge/automerge-rs/blob/d7d2916acb17d23d02ae249763aa0cf2f293d880/rust/automerge/src/columnar/encoding/rle.rs
use crate::{
    column::rle::Rleable,
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
        println!("buf {:?}", &buf);
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
        println!("{:?}", buf);
        let mut delta_rle_decoder = DeltaRleDecoder::new(&buf);
        let values: Vec<u64> = delta_rle_decoder.decode().unwrap();
        assert_eq!(values, vec![1, 2, 3, 4, 5, 6]);
    }
}
