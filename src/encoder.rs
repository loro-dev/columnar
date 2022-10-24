use std::{error::Error, fmt::Display};

use crate::{Encoder, encode::encode::{StructEncoder, ColumnEncoder}, Columns};


pub struct LoroEncoder;
#[derive(Debug)]
pub enum EncodeError {
    NotImplemented,
}
impl Error for EncodeError {}
impl Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not implemented")
    }
}

impl Encoder for LoroEncoder{
    type Ok = ();

    type Error = EncodeError;

    type StructEncoder = LoroStructEncoder;

    type ColumnEncoder = LoroColumnEncoder;

    fn encode_plain<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn encode_column_oriented<T: crate::ColumnOriented<C>, C: crate::Columns>(&mut self, value: &T) -> Result<Self::Ok, Self::Error> {
        let column_data = value.get_column_data();
        let mut column_encoder = self.encode_columns(&column_data);  
        Ok(())
    }

    fn encode_struct<T: crate::Encodable>(&mut self, value: &T) -> Result<Self::StructEncoder, Self::Error> {
        todo!()
    }

    fn encode_columns<C: Columns>(&mut self, columns: &C) -> Result<Self::ColumnEncoder, Self::Error>{
        todo!()
    }
}   

pub struct LoroStructEncoder;
pub struct LoroColumnEncoder;

impl StructEncoder for LoroStructEncoder {
    type Ok = ();

    type Error = EncodeError;

    fn encode_field<T: crate::Encodable>(&mut self, index: u32, value: &T) -> Result<(), Self::Error> {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ColumnEncoder for LoroColumnEncoder {
    type Ok = ();

    type Error = EncodeError;

    fn encode_column(&mut self, value: &crate::Column) -> Result<(), Self::Error> {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
    
}