use crate::{columnar::RleEncoder, ColumnarError, Columns};

use super::Columnar;


pub struct ColumnarRleEncoder<'c>{
    encoder: Columnar<'c>,
    buf: Vec<u8>,
    last: Option<u8>,
    count: u32,
}



impl<'r, T> RleEncoder<T> for ColumnarRleEncoder<'r> {
    type OK=();

    type Error=ColumnarError;

    fn encode<'c>(&self, data: &Vec<T>) -> Result<Self::OK, Self::Error> {
        todo!()
    }

    
}

