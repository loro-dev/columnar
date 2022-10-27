use std::fs::read;


pub struct ColumnarDecoder{
    scratch: Vec<u8>,
}

impl ColumnarDecoder{
    pub fn new() -> Self{
        Self{
            scratch: Vec::new(),
        }
    }
}