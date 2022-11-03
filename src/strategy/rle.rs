use crate::{
    columnar::{ColumnarDecoder, ColumnarEncoder},
    ColumnarError,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
/// Reference automerge implementation:
/// https://github.com/automerge/automerge-rs/blob/d7d2916acb17d23d02ae249763aa0cf2f293d880/rust/automerge/src/columnar/encoding/rle.rs
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::DerefMut};

pub trait Rle {
    type Value: PartialEq + Clone + Serialize;
    fn append<BT: Borrow<Self::Value>>(&mut self, value: BT) -> Result<(), ColumnarError>;
    fn finish(self) -> Result<(), ColumnarError>;
}

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
}

impl<'a> Rle for BoolRleEncoder<'a> {
    type Value = bool;

    fn append<BT: Borrow<Self::Value>>(&mut self, value: BT) -> Result<(), ColumnarError> {
        if *value.borrow() == self.last {
            self.count += 1;
        } else {
            self.count.serialize(self.ser.deref_mut());
            self.last = *value.borrow();
            self.count = 1;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(), ColumnarError> {
        if self.count > 0 {
            self.count.serialize(self.ser.deref_mut()).unwrap();
        }
        Ok(())
    }
}

pub struct AnyRleEncoder<'a, T> {
    ser: &'a mut ColumnarEncoder,
    state: RleState<T>,
}

impl<'a, T> Rle for AnyRleEncoder<'a, T>
where
    T: Clone + PartialEq + Serialize,
{
    type Value = T;

    fn append<BT: Borrow<Self::Value>>(&mut self, value: BT) -> Result<(), ColumnarError> {
        self.append_value(value)
    }

    fn finish(mut self) -> Result<(), ColumnarError> {
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
}

impl<'a, T> AnyRleEncoder<'a, T>
where
    T: PartialEq + Clone + Serialize,
{
    pub fn new(ser: &'a mut ColumnarEncoder) -> Self {
        Self {
            ser,
            state: RleState::Empty,
        }
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

pub(crate) trait DeRle {
    type Value;
    fn decode(&mut self) -> Result<Vec<Self::Value>, ColumnarError>;
    fn try_next(&mut self) -> Result<Option<Self::Value>, ColumnarError>;
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
    T: Clone + Deserialize<'de>,
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
}

impl<'a, 'de, T> DeRle for AnyRleDecoder<'a, 'de, T>
where
    T: Clone + Deserialize<'de>,
{
    type Value = T;

    fn decode(&mut self) -> Result<Vec<Self::Value>, ColumnarError> {
        let mut values = Vec::new();
        while let Some(value) = self.try_next()? {
            values.push(value);
        }
        Ok(values)
    }

    fn try_next(&mut self) -> Result<Option<Self::Value>, ColumnarError> {
        while self.count == 0 {
            // if self.de.is_empty() {
            //     return Ok(None);
            // }
            let count = isize::deserialize(self.de.deref_mut());
            if count.is_err() {
                return Ok(None);
            }
            let count = count.unwrap();
            if count > 0 {
                self.count = count;
                self.last_value = Some(T::deserialize(self.de.deref_mut())?);
                self.literal = false;
            } else if count < 0 {
                self.count = count.abs() as isize;
                self.literal = true;
            } else {
                return Err(ColumnarError::RleDecodeError("Invalid count".to_string()));
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

mod test {
    use crate::{
        columnar::{ColumnarDecoder, ColumnarEncoder},
        strategy::rle::{AnyRleDecoder, AnyRleEncoder, BoolRleEncoder, DeRle, Rle},
    };

    #[test]
    fn test_rle() {
        let mut columnar = ColumnarEncoder::new();
        let mut rle_encoder = AnyRleEncoder::<u64>::new(&mut columnar);
        rle_encoder.append(1000).unwrap();
        rle_encoder.append(1000).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.append(2).unwrap();
        rle_encoder.finish().unwrap();
        let mut buf = columnar.into_bytes();
        println!("buf {:?}", &buf);
        let mut columnar_decoder = ColumnarDecoder::new(&mut buf);
        let mut rle_decoder = AnyRleDecoder::<u64>::new(&mut columnar_decoder);
        assert_eq!(rle_decoder.decode().unwrap(), vec![1000, 1000, 2, 2, 2]);
    }

    #[test]
    fn test_bool_rle() {
        let mut buf = [0; 3];
        let mut columnar = ColumnarEncoder::new();
        let mut rle_encoder = BoolRleEncoder::new(&mut columnar);
        rle_encoder.append(true).unwrap();
        rle_encoder.append(true).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.append(false).unwrap();
        rle_encoder.finish().unwrap();
        assert_eq!(&buf, &[0, 2, 3]);
    }
}
