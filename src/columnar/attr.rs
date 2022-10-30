use crate::ColumnarError;

#[derive(Debug, Clone, PartialEq)]
pub enum Strategy {
    Rle,
    BoolRle,
    Delta,
    DeltaRle,
}

// TODO: use some other crate
impl Strategy {
    pub fn try_from_u8(v: u8) -> Result<Option<Self>, ColumnarError> {
        match v {
            0 => Ok(None),
            1 => Ok(Some(Strategy::Rle)),
            2 => Ok(Some(Strategy::BoolRle)),
            3 => Ok(Some(Strategy::Delta)),
            4 => Ok(Some(Strategy::DeltaRle)),
            _ => Err(ColumnarError::InvalidStrategy),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Strategy::Rle => 1,
            Strategy::BoolRle => 2,
            Strategy::Delta => 3,
            Strategy::DeltaRle => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnAttr {
    pub index: usize,
    pub strategy: Option<Strategy>,
}
