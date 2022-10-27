// #![feature(generic_const_exprs)]
mod columnar;
pub use columnar::{Column, ColumnAttr, ColumnData, ColumnOriented, Columns, Row, Strategy};
mod columnar_impl;
pub use columnar_impl::{ser, de};
mod err;
pub use err::ColumnarError;