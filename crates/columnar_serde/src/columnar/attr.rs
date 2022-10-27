#[derive(Debug, Clone)]
pub enum Strategy {
    Plain,
    BoolRle,
    Rle,
}

#[derive(Debug)]
pub struct ColumnAttr{
    pub index: usize,
    pub strategies: Vec<Strategy>,
}