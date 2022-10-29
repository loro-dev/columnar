#[derive(Debug, Clone, PartialEq)]
pub enum Strategy {
    Plain,
    BoolRle,
    Rle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnAttr {
    pub index: usize,
    pub strategies: Vec<Strategy>,
}
