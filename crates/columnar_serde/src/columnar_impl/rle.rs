/// Reference automerge implementation: 
/// https://github.com/automerge/automerge-rs/blob/d7d2916acb17d23d02ae249763aa0cf2f293d880/rust/automerge/src/columnar/encoding/rle.rs

// TODO: add rle trait, to impl num and bool 

use std::{borrow::Borrow};

use crate::ColumnData;


pub struct RleEncoder<T>{
    encoder: T,
}

impl<T> RleEncoder<T> where T: Rle{
    pub fn new(rle: T) -> Self{
        Self{
            encoder: rle
        }
    }

    pub fn append<BT: Borrow<T::Value>>(&mut self, value: BT){
        self.encoder.append(value)
    }
    pub fn finish(mut self) -> Vec<T::Output>{
        self.encoder.finish()
    }
}


pub trait Rle {
    type Value: PartialEq + Clone;
    type Output;
    fn append<BT: Borrow<Self::Value>>(&mut self, value: BT);
    fn finish(self) -> Vec<Self::Output>;
}

pub struct BoolRleEncoder{
    buf: Vec<usize>,
    last: bool,
    count: usize,
}

impl BoolRleEncoder {
    fn new() -> Self{
        Self{
            buf: Vec::new(),
            last: false,
            count: 0,
        }
    }
}

impl Rle for BoolRleEncoder{
    type Value = bool;
    type Output = usize;

    fn append<BT: Borrow<Self::Value>>(&mut self, value: BT){
        if *value.borrow() == self.last {
            self.count += 1;
        } else {
            self.buf.push(self.count as usize);
            self.last = *value.borrow();
            self.count = 1;
        }
    }

    fn finish(mut self) -> Vec<Self::Output>{
        if self.count > 0 {
            self.buf.push(self.count as usize);
        }
        self.buf
    }
}

pub struct AnyRleEncoder<T>{
    buf: Vec<RleData<T>>,
    state: RleState<T>,
}


impl<T> Rle for AnyRleEncoder<T> where T: Clone + PartialEq{
    type Value=Option<T>;

    type Output = RleData<T>;

    fn append<BT: Borrow<Self::Value>>(&mut self, value: BT) {
        match value.borrow() {
            Some(t) => self.append_value(t),
            None => self.append_null(),
        }
    }

    fn finish(mut self) -> Vec<Self::Output> {
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
        self.buf
    }
}


impl<T> AnyRleEncoder<T> where T: PartialEq + Clone{
    pub fn new() -> Self{
        Self{
            buf: Vec::new(),
            state: RleState::Empty,
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
        self.encode_length(len as isize);
        self.encode_content(val.clone());
    }

    fn flush_null_run(&mut self, len: usize) {
        // TODO: check if this is correct
        self.encode_length(0);
        self.encode_length(len as isize);
    }

    fn flush_lit_run(&mut self, run: Vec<T>) {
        self.encode_length(-(run.len() as isize));
        for val in run {
            self.encode_content(val);
        }
    }

    fn encode_content(&mut self, val: T) {
        self.buf.push(RleData::Content(val));
    }

    fn encode_length(&mut self, val: isize) {
        self.buf.push(RleData::Length(val));
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

// TODO: consider using ColumnData?
#[derive(Debug, PartialEq)]
pub enum RleData<T>{
    Content(T),
    Length(isize)
}


mod test{
    use crate::columnar_impl::rle::{RleEncoder, AnyRleEncoder, BoolRleEncoder, RleData};


    #[test]
    fn test_rle(){
        let mut rle_encoder = RleEncoder::new(AnyRleEncoder::<u64>::new());
        rle_encoder.append(Some(1000));
        rle_encoder.append(Some(1000));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(2));
        rle_encoder.append(Some(2));
        let data = rle_encoder.finish();
        println!("{:?}", data);
        assert_eq!(data, vec![
            RleData::Length(2),
            RleData::Content(1000),
            RleData::Length(3),
            RleData::Content(2),
        ]);
    }

    #[test]
    fn test_bool_rle(){
        let mut rle_encoder = RleEncoder::new(BoolRleEncoder::new());
        rle_encoder.append(true);
        rle_encoder.append(true);
        rle_encoder.append(false);
        rle_encoder.append(false);
        rle_encoder.append(false);
        let data = rle_encoder.finish();
        assert_eq!(data, vec![0, 2, 3]);
    }
}