use serde::{Serialize, Serializer};

use crate::{Columns, ColumnarError, columnar_impl::rle::RleEncoder};

use super::columnar::Columnar;


pub struct ColumnarEncoder{
    buf: Vec<u8>,
    // ColumnEncoder 中的 serde
    ser: Columnar,
}

impl<'c> ColumnarEncoder{
    pub fn new() -> Self{
        Self{
            ser: Columnar::new(),
            buf: Vec::new()
        }
    }

    pub fn encode(&mut self, data: &Columns<'c>) -> Result<Vec<u8>, ColumnarError>{
        todo!()
    }

    fn column_as_bytes(&self) -> Vec<u8>{
       todo!()
    }
}