#![feature(generic_const_exprs)]
mod object;
mod encode;
pub use encode::{Strategy, ColumnAttr, Columns, ColumnOriented, Row, encode::{Encodable, Encoder}, Column, ColumnTrait};
mod encoder;