use std::error::Error;

use crate::columnar::{ColumnOriented, Columns, Column};

pub trait Encodable {
    fn encode<E>(&self, encoder: &mut E) -> Result<E::Ok, E::Error> where E: Encoder;
}

pub trait Encoder {
    type Ok;
    type Error: Error;
    type StructEncoder: StructEncoder;
    type ColumnEncoder: ColumnEncoder;

    // Base types
    // fn encode_string(&mut self, value: &String) -> Result<Self::Ok, Self::Error>{
    //     self.encode_str(value.as_str())
    // }
    // fn encode_str(&mut self, value: &str) -> Result<Self::Ok, Self::Error>;
    // fn encode_u64(&mut self, value: u64) -> Result<Self::Ok, Self::Error>;

    fn encode_plain<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>;
    fn encode_column_oriented<T: ColumnOriented<C>, C: Columns>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>;
    fn encode_struct<T: Encodable>(&mut self, value: &T) -> Result<Self::StructEncoder, Self::Error>;

    // encode a column by attr
    fn encode_columns<C: Columns>(&mut self, columns: &C) -> Result<Self::ColumnEncoder, Self::Error>;
}

pub trait StructEncoder {
    type Ok;
    type Error: Error;

    fn encode_field<T: Encodable>(&mut self, index: u32, value: &T) -> Result<(), Self::Error>;
    fn end(self) -> Result<Self::Ok, Self::Error>;
    
}

pub trait ColumnEncoder {
    type Ok;
    type Error: Error;
    type RleEncoder: RleEncoder;

    fn encode_rle(&mut self, value: &Column) -> Result<Self::RleEncoder, Self::Error>;
    fn end(self) -> Result<Self::Ok, Self::Error>;
    
}

pub trait RleEncoder{ 
    type Ok;
    type Error: Error;

    fn encode_column(&mut self, value: &Column) -> Result<(), Self::Error>;
    fn end(self) -> Result<Self::Ok, Self::Error>;
}