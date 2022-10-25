use std::{error::Error, fmt::Display};

use crate::{Encoder, encode::encode::{StructEncoder, ColumnEncoder}, Columns};




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

