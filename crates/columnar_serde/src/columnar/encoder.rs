use crate::{Column, ColumnData, Columns};

pub trait ColumnEncoder {
    type OK;
    type Error;
    type RleEncoder: Rle;
    fn encode_rle(&self, strategy: &Strategy) -> Result<Self::RleEncoder, Self::Error>;
}

#[derive(Debug, Clone)]
pub enum Strategy {
    Plain,
    BoolRle,
    Rle,
}

pub trait Rle {
    type OK;
    type Error;
    fn encode<T>(&self, data: &Vec<T>) -> Result<Self::OK, Self::Error>;
}
