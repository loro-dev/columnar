use crate::Strategy;

#[derive(Debug)]
pub struct ColumnAttr{
    pub index: usize,
    pub strategies: Vec<Strategy>,
}