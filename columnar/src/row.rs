use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

pub trait VecRow: Sized {
    const FIELD_NUM: usize;
    fn serialize_columns<S>(rows: &Vec<Self>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize_columns<'de, D>(de: D) -> Result<Vec<Self>, D::Error>
    where
        D: serde::Deserializer<'de>;
}

pub trait MapRow<'de>: Sized {
    const FIELD_NUM: usize;
    type Key: Serialize + Deserialize<'de>;
    fn serialize_columns<S>(rows: &HashMap<Self::Key, Self>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize_columns<D>(de: D) -> Result<HashMap<Self::Key, Self>, D::Error>
    where
        D: serde::Deserializer<'de>;
}
