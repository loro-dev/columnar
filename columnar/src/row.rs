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

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnarVec<'c, T: VecRow + Clone>(pub Cow<'c, Vec<T>>);

impl<'c, T: VecRow + Clone> ColumnarVec<'c, T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self(Cow::Owned(vec))
    }

    pub fn from_borrowed(vec: &'c Vec<T>) -> Self {
        Self(Cow::Borrowed(vec))
    }
}

impl<'c, T: VecRow + Clone> Into<Vec<T>> for ColumnarVec<'c, T> {
    fn into(self) -> Vec<T> {
        self.0.into_owned()
    }
}

impl<'c, T: VecRow + Clone> Serialize for ColumnarVec<'c, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        VecRow::serialize_columns(&self.0, serializer)
    }
}

impl<'de, 'c, T: VecRow + Clone> Deserialize<'de> for ColumnarVec<'c, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ColumnarVec(Cow::Owned(VecRow::deserialize_columns(
            deserializer,
        )?)))
    }
}
