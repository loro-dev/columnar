use serde::{Serialize, Serializer};

use crate::{Columns, ColumnarError, columnar_impl::rle::RleEncoder, Column, ColumnData, Strategy};

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

    pub fn encode(&mut self, columns: &Columns<'c>) -> Result<(), ColumnarError>{
        for column in columns.iter() {
            self.encode_column(column)?;
        }
        Ok(())
    }

    fn encode_column(&mut self, column: &Column<'c>) -> Result<(), ColumnarError>{
        let Column(data, attr) = column;
        let mut strategies = attr.strategies.clone();
        while let Some(strategy) = strategies.pop(){
            match strategy{
                Strategy::Plain => {
                    self.encode_plain(data)?;
                }
                Strategy::RLE => {
                    self.encode_rle(data)?;
                }
            }
        }
        Ok(())
    }

    fn encode_rle(&mut self, column_data: &Vec<ColumnData<'c>>) -> Result<Vec<ColumnData>, ColumnarError>{
        todo!()
        // let mut rle = RleEncoder::<ColumnData>::new(&mut self.ser);
        // for column_data in column_data.iter(){
        //     rle.append(Some(column_data));
        // }
        // Ok(rle.finish().into())
    }

    fn encode_plain(&mut self, column_data: &Vec<ColumnData<'c>>) -> Result<(), ColumnarError>{
        // let seq = self.ser.serialize_seq(Some(column_data.len()))?;
        // for column_data in column_data.iter(){
        //     seq.serialize_element(column_data)?;
        // }
        Ok(())
    }

    pub(crate) fn finish(self) -> Vec<u8>{
        todo!()
        // TODO:
        // self.ser.buf()
    }
}