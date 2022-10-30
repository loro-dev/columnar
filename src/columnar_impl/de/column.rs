use crate::{CellData, Column, ColumnAttr, ColumnarError, Columns, Strategy};

use super::columnar::ColumnarDeserializer;
use serde::de::Deserialize;

pub struct ColumnDecoder<'de> {
    // ColumnDecoder 中的 serde
    de: ColumnarDeserializer<'de>,
}

impl<'de: 'c, 'c> ColumnDecoder<'de> {
    pub(crate) fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: ColumnarDeserializer::new(bytes),
        }
    }

    pub(crate) fn decode(&mut self) -> Result<Columns<'c>, ColumnarError> {
        let column_len = Deserialize::deserialize(&mut self.de)?;
        let mut columns = Vec::with_capacity(column_len);
        for index in 0..column_len {
            columns.push(self.decode_column(index)?);
        }
        Ok(Columns(columns))
    }

    fn decode_column(&mut self, index: usize) -> Result<Column<'c>, ColumnarError> {
        let strategy = Strategy::try_from_u8(u8::deserialize(&mut self.de)?)?;
        let mut cells_data = self.decode_plain()?;
        if let Some(strategy) = &strategy {}

        Ok(Column(cells_data, ColumnAttr { index, strategy }))
    }

    fn decode_plain(&mut self) -> Result<Vec<CellData<'c>>, ColumnarError> {
        let len = usize::deserialize(&mut self.de)?;
        let mut cells_data = Vec::with_capacity(len);
        for _ in 0..len {
            let cell_data = CellData::deserialize(&mut self.de)?;
            cells_data.push(cell_data);
        }
        Ok(cells_data)
    }

    fn decode_rle(&self, cells_data: Vec<CellData>) -> Result<Vec<CellData>, ColumnarError> {
        todo!()
    }

    fn decode_bool_rle(&self, cells_data: Vec<CellData>) -> Result<Vec<CellData>, ColumnarError> {
        todo!()
    }

    fn decode_delta(&self, cells_data: Vec<CellData>) -> Result<Vec<CellData>, ColumnarError> {
        todo!()
    }
}
