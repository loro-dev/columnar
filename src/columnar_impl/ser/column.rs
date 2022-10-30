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
        self.ser.serialize_u64(columns.len() as u64)?;
        for column in columns.iter() {
            self.encode_column(column)?;
        }
        Ok(())
    }

    fn encode_column(&mut self, column: &Column<'c>) -> Result<(), ColumnarError> {
        let Column(data, attr) = column;
        let mut owned: Vec<CellData>;
        let mut tmp_data = data;
        // serialize strategy type
        if let Some(strategy) = &attr.strategy {
            self.ser.serialize_u8(strategy.as_u8())?;
            match strategy {
                Strategy::Rle => {
                    owned = self.encode_rle(tmp_data)?;
                    tmp_data = &owned;
                }
                Strategy::BoolRle => {
                    owned = self.encode_rle(tmp_data)?;
                    tmp_data = &owned;
                }
                Strategy::Delta => {
                    owned = self.encode_delta(tmp_data)?;
                    tmp_data = &owned;
                }
                Strategy::DeltaRle => {
                    owned = self.encode_delta(tmp_data)?;
                    owned = self.encode_rle(&owned)?;
                    tmp_data = &owned;
                }
            }
        } else {
            self.ser.serialize_u8(0)?;
        }
        self.encode_plain(tmp_data)
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

    fn encode_delta(
        &self,
        column_data: &Vec<CellData<'c>>,
    ) -> Result<Vec<CellData<'c>>, ColumnarError> {
        todo!()
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

    fn encode_plain(&mut self, cells_data: &Vec<CellData<'c>>) -> Result<(), ColumnarError> {
        let mut seq = self.ser.serialize_seq(Some(cells_data.len()))?;
        for cell_data in cells_data.iter() {
            seq.serialize_element(cell_data)?;
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.ser.to_bytes()
    }
}
