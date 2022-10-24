mod data;
use data::ColumnData;
mod example;
pub mod encode;

pub enum Strategy{
    Plain,
    RLE,
    Delta,
    Dictionary,
    Leb128,
    ULeb128,
    ZigZag,
    BitPacking,
}

pub struct ColumnAttr {
    index: usize,
    strategies: Vec<Strategy>
}

pub trait Row{}

pub trait ColumnTrait {
    type Item;
}

pub struct Column{
    data: Vec<ColumnData>,
    attr: ColumnAttr,
}



pub trait Columns{
    type Row: Row;
    // const COLUMNS_ATTRS : [ColumnAttr; N];
    // const ATTR_NUM: usize;
    // fn attr_num() -> usize;
    // fn column_attrs() -> Vec<ColumnAttr>;
    fn column_data(&self) -> Vec<Column>;
}

pub trait ColumnOriented<T: Columns> {
    fn get_column_data(&self) -> T;
}


