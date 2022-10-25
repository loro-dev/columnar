use crate::{ColumnData, Columns, Column};


pub trait ColumnarEncoder{
    type OK;
    type Error;
    type RleEncoder;
    fn encode_rle(&self, strategy: &Strategy) -> Result<Self::RleEncoder, Self::Error>;
}

#[derive(Debug)]
pub enum Strategy{
    Plain,
    RLE
}

pub trait RleEncoder<T>{
    type OK;
    type Error;
    fn encode<'c>(&self, data: &Vec<T>) -> Result<Self::OK, Self::Error>;
}