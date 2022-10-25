use columnar::{Encoder, StructEncoder, ColumnEncoder, Column, Encodable, RleEncoder};
mod error;
use self::error::EncodeError;

pub struct LoroEncoder;

impl LoroEncoder{
    pub fn new() -> Self{
        Self{}
    }
}

impl Encoder for LoroEncoder {
    type Ok = Vec<u8>;
    type Error = EncodeError;

    type StructEncoder = LoroStructEncoder;

    type ColumnEncoder = LoroColumnEncoder;

    fn encode_plain<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn encode_column_oriented<T: columnar::ColumnOriented<C>, C: columnar::Columns>(&mut self, value: &T) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn encode_struct<T: columnar::Encodable>(&mut self, value: &T) -> Result<Self::StructEncoder, Self::Error> {
        todo!()
    }

    fn encode_columns<C: columnar::Columns>(&mut self, columns: &C) -> Result<Self::ColumnEncoder, Self::Error> {
        todo!()
    }
}

pub struct LoroStructEncoder;
pub struct LoroColumnEncoder;
pub struct LoroRleEncoder;

impl StructEncoder for LoroStructEncoder {
    type Ok = ();

    type Error = EncodeError;

    fn encode_field<T: Encodable>(&mut self, index: u32, value: &T) -> Result<(), Self::Error> {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ColumnEncoder for LoroColumnEncoder {
    type Ok = ();

    type Error = EncodeError;

    type RleEncoder = LoroRleEncoder;

    fn encode_rle(&mut self, value: &Column) -> Result<Self::RleEncoder, Self::Error> {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl RleEncoder for LoroRleEncoder {
    type Ok= ();

    type Error=EncodeError;

    fn encode_column(&mut self, value: &Column) -> Result<(), Self::Error> {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}