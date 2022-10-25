use crate::Strategy;

#[derive(Debug)]
pub struct ColumnAttr{
    pub index: usize,
    pub strategy: Strategy,
}