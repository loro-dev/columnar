use std::collections::HashMap;

#[derive(Clone)]
pub enum ColumnData {
    String(String),
    U64(u64),
    I64(i64),
    F64(f64),
    Vec(Vec<ColumnData>),
    HashMap(HashMap<ColumnData, ColumnData>),
}