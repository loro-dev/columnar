use std::borrow::Cow;

use crate::data::ColumnData;


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
    pub index: usize,
    pub strategies: Vec<Strategy>
}

pub trait Row{}

pub trait ColumnTrait {
    type Item;
}

pub struct Column{
    pub data: Vec<ColumnData>,
    pub attr: ColumnAttr,
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