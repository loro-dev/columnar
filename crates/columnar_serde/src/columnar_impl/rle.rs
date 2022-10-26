use std::{borrow::Borrow, ptr::NonNull};

/// Reference automerge implementation: 
/// https://github.com/automerge/automerge-rs/blob/d7d2916acb17d23d02ae249763aa0cf2f293d880/rust/automerge/src/columnar/encoding/rle.rs

use serde::{Serialize, Serializer, ser::SerializeSeq};

use super::{ColumnarEncoder, columnar::Columnar};

pub struct RleEncoder<'a, T: Serialize>{
    ser: &'a mut Columnar,
    state: RleState<T>,
    start: usize,
    written: usize,
}

impl<'a, T> RleEncoder<'a, T> where T: Serialize + PartialEq + Clone{
    pub fn new(ser: &'a mut Columnar) -> Self{
        let start = ser.buf().len();
        Self{
            ser,
            state: RleState::Empty,
            written: 0,
            start
        }
    }

    pub(crate) fn append_value<BT: Borrow<T>>(&mut self, value: BT) {
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
            RleState::NullRun(size) | RleState::InitialNullRun(size) => {
                self.flush_null_run(size);
                RleState::LoneVal(value.borrow().clone())
            }
        }
    }

    pub fn append_null(&mut self){
        self.state = match self.take_state() {
            RleState::Empty => RleState::InitialNullRun(1),
            RleState::InitialNullRun(size) => RleState::InitialNullRun(size + 1),
            RleState::NullRun(size) => RleState::NullRun(size + 1),
            RleState::LoneVal(other) => {
                self.flush_lit_run(vec![other]);
                RleState::NullRun(1)
            }
            RleState::Run(other, len) => {
                self.flush_run(&other, len);
                RleState::NullRun(1)
            }
            RleState::LiteralRun(last, mut run) => {
                run.push(last);
                self.flush_lit_run(run);
                RleState::NullRun(1)
            }
        }
    }

    pub fn append<BT: Borrow<T>>(&mut self, value: Option<BT>){
        match value {
            Some(t) => self.append_value(t),
            None => self.append_null(),
        }
    }

    fn take_state(&mut self) -> RleState<T> {
        let mut state = RleState::Empty;
        std::mem::swap(&mut self.state, &mut state);
        state
    }

    fn flush_run(&mut self, val: &T, len: usize) {
        self.encode(&(len as i64));
        self.encode(val);
    }

    fn flush_null_run(&mut self, len: usize) {
        self.encode::<i64>(&0);
        self.encode(&len);
    }

    fn flush_lit_run(&mut self, run: Vec<T>) {
        self.encode(&-(run.len() as i64));
        for val in run {
            self.encode(&val);
        }
    }

    /// Flush the encoded values and return the output buffer and the number of bytes written
    pub(crate) fn finish(mut self) -> usize {
        match self.take_state() {
            RleState::InitialNullRun(_size) => {}
            RleState::NullRun(size) => {
                self.flush_null_run(size);
            }
            RleState::LoneVal(value) => self.flush_lit_run(vec![value]),
            RleState::Run(value, len) => self.flush_run(&value, len),
            RleState::LiteralRun(last, mut run) => {
                run.push(last);
                self.flush_lit_run(run);
            }
            RleState::Empty => {}
        };
        self.ser.buf().len() - self.start
    }

    fn encode<V>(&mut self, val: &V)
    where
        V: Serialize,
    {
        val.serialize(&mut *self.ser).unwrap();
    }
}


enum RleState<T> {
    Empty,
    // Note that this is different to a `NullRun` because if every element of a column is null
    // (i.e. the state when we call `finish` is `InitialNullRun`) then we don't output anything at
    // all for the column
    InitialNullRun(usize),
    NullRun(usize),
    LiteralRun(T, Vec<T>),
    LoneVal(T),
    Run(T, usize),
}


mod test{

    use crate::columnar_impl::columnar::Columnar;

    use super::RleEncoder;

    #[test]
    fn test_rle(){
        let mut columnar = Columnar::new();
        let mut rle_encoder = RleEncoder::<u64>::new(&mut columnar);
        rle_encoder.append(Some(1000));
        rle_encoder.append(Some(1000));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(2));
        let len = rle_encoder.finish();
        println!("{:?}", columnar);
        assert_eq!(columnar.buf(), &vec![2 as u8,232, 7, 3, 2]);
        assert_eq!(columnar.buf().len(), len);
    }

    #[test]
    fn test_rle2(){
        let mut columnar = Columnar::new();
        let mut rle_encoder = RleEncoder::<u64>::new(&mut columnar);
        rle_encoder.append(Some(1));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(3));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(3));
        rle_encoder.finish();
        println!("{:?}", columnar);
        // assert_eq!(columnar.buf(), &vec![2 as u64,1,3,2]);
    }
}