use std::collections::HashMap;

use crate::{Encodable, Encoder};
pub enum ColumnData {
    String(String),
    U64(u64),
    I64(i64),
    F64(f64),
    Vec(Vec<ColumnData>),
    HashMap(HashMap<ColumnData, ColumnData>),
}

// impl Encodable for ColumnData{
//     fn encode<E>(&self, encoder: &mut E) -> Result<E::Ok, E::Error> where E: Encoder{
//         match self {
//             ColumnData::String(x) => {
//                 encoder.encode_string(x)
//             }
//             ColumnData::U64(x) => {
//                 encoder.encode_u64(*x)
//             }
//             ColumnData::I64(x) => {
//                 encoder.encode_i64(*x)
//             }
//             ColumnData::F64(x) => {
//                 encoder.encode_f64(*x)
//             }
//             ColumnData::Vec(x) => {
//                 encoder.encode_vec(x)
//             }
//             ColumnData::HashMap(x) => {
//                 encoder.encode_hashmap(x)
//             }
//         }
//     }
// }