use serde::{ser::SerializeSeq, Serializer};

use crate::{
    columnar_impl::ser::rle::RleEncoder, CellData, Column, ColumnarError, Columns, Strategy,
};

use super::{
    columnar::ColumnarSerializer,
    rle::{AnyRleEncoder, BoolRleEncoder},
};

pub struct ColumnEncoder {
    // ColumnEncoder 中的 serde
    ser: ColumnarSerializer,
}

impl<'c> ColumnEncoder {
    pub fn new() -> Self {
        Self {
            ser: ColumnarSerializer::new(),
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
        let mut owned: Vec<CellData>;
        let mut tmp_data = data;
        while let Some(strategy) = strategies.pop() {
            match strategy {
                Strategy::Plain => {
                    self.encode_plain(tmp_data)?;
                    break;
                }
                Strategy::Rle => {
                    owned = self.encode_rle(tmp_data)?;
                    tmp_data = &owned;
                }
                Strategy::BoolRle => {
                    owned = self.encode_rle(tmp_data)?;
                    tmp_data = &owned;
                }
            }
        }
        Ok(())
    }

    fn encode_rle(
        &self,
        column_data: &Vec<CellData<'c>>,
    ) -> Result<Vec<CellData<'c>>, ColumnarError> {
        let mut rle_encoder = RleEncoder::new(AnyRleEncoder::<CellData>::new());
        for data in column_data.into_iter() {
            rle_encoder.append(data);
        }
        let rle_encoded = rle_encoder.finish().into_iter().map(|r| r.into()).collect();
        Ok(rle_encoded)
    }

    fn encode_bool_rle(
        &self,
        column_data: &Vec<CellData<'c>>,
    ) -> Result<Vec<CellData>, ColumnarError> {
        let mut rle_encoder = RleEncoder::new(BoolRleEncoder::new());
        for data in column_data.into_iter() {
            if let CellData::Bool(b) = data {
                rle_encoder.append(b);
            } else {
                return Err(ColumnarError::InvalidDataType);
            }
        }
        let rle_encoded = rle_encoder
            .finish()
            .into_iter()
            .map(|r| CellData::U64(r as u64))
            .collect();
        Ok(rle_encoded)
    }

    fn encode_plain(&mut self, column_data: &Vec<CellData<'c>>) -> Result<(), ColumnarError> {
        let mut seq = self.ser.serialize_seq(Some(column_data.len()))?;
        for column_data in column_data.iter() {
            seq.serialize_element(column_data)?;
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.ser.to_bytes()
    }
}
