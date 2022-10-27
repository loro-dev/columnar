use serde::{Serialize, Serializer};

use crate::{columnar_impl::rle::RleEncoder, Column, ColumnData, ColumnarError, Columns, Strategy};

use super::{columnar::Columnar, rle::AnyRleEncoder};

pub struct ColumnarEncoder {
    buf: Vec<u8>,
    // ColumnEncoder 中的 serde
    ser: Columnar,
}

impl<'c> ColumnarEncoder {
    pub fn new() -> Self {
        Self {
            ser: Columnar::new(),
            buf: Vec::new(),
        }
    }

    pub fn encode(&mut self, columns: &Columns<'c>) -> Result<(), ColumnarError> {
        for column in columns.iter() {
            self.encode_column(column)?;
        }
        Ok(())
    }

    fn encode_column(&mut self, column: &Column<'c>) -> Result<(), ColumnarError> {
        let Column(data, attr) = column;
        let mut strategies: Vec<&Strategy> = attr.strategies.iter().rev().collect();
        while let Some(strategy) = strategies.pop() {
            match strategy {
                Strategy::Plain => {
                    self.encode_plain(data)?;
                }
                Strategy::Rle => {
                    self.encode_rle(data)?;
                }
                Strategy::BoolRle => {
                    self.encode_bool_rle(data)?;
                }
            }
        }
        Ok(())
    }

    fn encode_rle(
        &mut self,
        column_data: &Vec<ColumnData<'c>>,
    ) -> Result<Vec<ColumnData<'c>>, ColumnarError> {
        let mut rle_encoder = RleEncoder::new(AnyRleEncoder::<ColumnData>::new());
        for data in column_data.into_iter() {
            rle_encoder.append(data);
        }
        let rle_encoded = rle_encoder.finish().into_iter().map(|r| r.into()).collect();
        Ok(rle_encoded)
    }

    fn encode_bool_rle(
        &mut self,
        column_data: &Vec<ColumnData<'c>>,
    ) -> Result<Vec<ColumnData>, ColumnarError> {
        todo!()
    }

    fn encode_plain(&mut self, column_data: &Vec<ColumnData<'c>>) -> Result<(), ColumnarError> {
        // let seq = self.ser.serialize_seq(Some(column_data.len()))?;
        // for column_data in column_data.iter(){
        //     seq.serialize_element(column_data)?;
        // }
        Ok(())
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        todo!()
        // TODO:
        // self.ser.buf()
    }
}
