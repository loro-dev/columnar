/// Reference automerge implementation:
/// https://github.com/automerge/automerge-rs/blob/d7d2916acb17d23d02ae249763aa0cf2f293d880/rust/automerge/src/columnar/encoding/rle.rs
use crate::{
    column::rle::Rleable,
    columnar_internal::{ColumnarDecoder, ColumnarEncoder},
    ColumnarError,
};
use serde::{Deserialize, Serialize};

use std::{borrow::Borrow, marker::PhantomData, ops::DerefMut};

use super::MAX_RLE_COUNT;

pub struct BoolRleEncoder<'a> {
    ser: &'a mut ColumnarEncoder,
    last: bool,
    count: usize,
}

impl<'a> BoolRleEncoder<'a> {
    pub(crate) fn new(ser: &'a mut ColumnarEncoder) -> Self {
        Self {
            ser,
            last: false,
            count: 0,
        }
    }

    pub(crate) fn append(&mut self, value: bool) -> Result<(), ColumnarError> {
        if *value.borrow() == self.last {
            self.count += 1;
        } else {
            self.count.serialize(self.ser.deref_mut())?;
            self.last = *value.borrow();
            self.count = 1;
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<(), ColumnarError> {
        if self.count > 0 {
            self.count.serialize(self.ser.deref_mut()).unwrap();
        }
        Ok(())
    }
}

pub(crate) struct AnyRleEncoder<'a, T> {
    ser: &'a mut ColumnarEncoder,
    state: RleState<T>,
}

impl<'a, T> AnyRleEncoder<'a, T>
where
    T: Rleable,
{
    pub fn new(ser: &'a mut ColumnarEncoder) -> Self {
        Self {
            ser,
            state: RleState::Empty,
        }
    }

    pub(crate) fn append<BT: Borrow<T>>(&mut self, value: BT) -> Result<(), ColumnarError> {
        self.append_value(value)
    }

    pub(crate) fn finish(mut self) -> Result<(), ColumnarError> {
        match self.take_state() {
            RleState::LoneVal(value) => self.flush_lit_run(vec![value]),
            RleState::Run(value, len) => self.flush_run(&value, len),
            RleState::LiteralRun(last, mut run) => {
                run.push(last);
                self.flush_lit_run(run);
            }
            RleState::Empty => {}
        };
        Ok(())
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

enum RleState<T> {
    Empty,
    LiteralRun(T, Vec<T>),
    LoneVal(T),
    Run(T, usize),
}

pub(crate) struct AnyRleDecoder<'a, 'de, T> {
    de: &'a mut ColumnarDecoder<'de>,
    last_value: Option<T>,
    count: isize,
    literal: bool,
    lifetime: PhantomData<&'de ()>,
}

impl<'a, 'de, T> AnyRleDecoder<'a, 'de, T>
where
    T: Rleable,
{
    pub(crate) fn new(de: &'a mut ColumnarDecoder<'de>) -> Self {
        Self {
            de,
            last_value: None,
            count: 0,
            literal: false,
            lifetime: PhantomData,
        }
    }

    pub(crate) fn decode(&mut self) -> Result<Vec<T>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<T>, ColumnarError> {
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
                    self.count = n.abs() as isize;
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

pub(crate) struct BoolRleDecoder<'a, 'de> {
    de: &'a mut ColumnarDecoder<'de>,
    last_value: bool,
    count: usize,
}

impl<'a, 'de> BoolRleDecoder<'a, 'de> {
    pub(crate) fn new(de: &'a mut ColumnarDecoder<'de>) -> Self {
        Self {
            de,
            last_value: true,
            count: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn decode(&mut self) -> Result<Vec<bool>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    fn try_next(&mut self) -> Result<Option<bool>, ColumnarError> {
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

mod test {

    #[test]
    fn test_rle() {
        use super::*;
        let mut columnar = ColumnarEncoder::new();
        let mut rle_encoder = AnyRleEncoder::<u64>::new(&mut columnar);
        rle_encoder.append(1000).unwrap();
        rle_encoder.append(1000).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.finish().unwrap();
        let buf = columnar.into_bytes();
        println!("buf {:?}", &buf);
        let mut columnar_decoder = ColumnarDecoder::new(&buf);
        let mut rle_decoder = AnyRleDecoder::<u64>::new(&mut columnar_decoder);
        assert_eq!(rle_decoder.decode().unwrap(), vec![1000, 1000, 2, 2, 2]);
    }

    #[test]
    fn test_bool_rle() {
        use super::*;
        let mut columnar = ColumnarEncoder::new();
        let mut rle_encoder = BoolRleEncoder::new(&mut columnar);
        rle_encoder.append(true).unwrap();
        rle_encoder.append(true).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.finish().unwrap();
        let buf = columnar.into_bytes();
        assert_eq!(&buf, &[0, 2, 3]);
        let mut columnar_decoder = ColumnarDecoder::new(buf.as_slice());
        let mut rle_decoder = BoolRleDecoder::new(&mut columnar_decoder);
        assert_eq!(
            rle_decoder.decode().unwrap(),
            vec![true, true, false, false, false]
        );
    }
}
