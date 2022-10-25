
use crate::{err::ColumnarError, columnar::ColumnarEncoder, Columns};

use super::rle::ColumnarRleEncoder;

pub enum ColumnarBuf<'c>{
    Encoding(Columns<'c>),
    End(Vec<u8>)
} 

pub struct Columnar<'c>{
    buf: Option<ColumnarBuf<'c>>
}

impl<'c> Columnar<'c>{
    pub fn new() -> Self{
        Self{
            buf: None
        }
    }

    pub fn encode(&mut self, data: &Columns<'c>) -> Result<Vec<u8>, ColumnarError>{
        todo!()
    }

    fn column_as_bytes(&self) -> Vec<u8>{
       todo!()
    }
}

impl<'c> ColumnarEncoder for Columnar<'c>{
    type OK=();

    type Error=ColumnarError;

    type RleEncoder = ColumnarRleEncoder<'c>;

    fn encode_rle(&self, strategy: &crate::Strategy) -> Result<Self::RleEncoder, Self::Error> {
        todo!()
    }
}