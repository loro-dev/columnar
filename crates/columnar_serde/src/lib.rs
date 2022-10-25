// #![feature(generic_const_exprs)]
mod columnar;
pub use columnar::{Column, ColumnAttr, ColumnData, ColumnOriented, Columns, Row, Strategy};
mod columnar_impl;
mod err;
pub use err::ColumnarError;
mod serde;