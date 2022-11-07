use std::borrow::Cow;

use serde::{Deserialize, Serialize};

pub trait VecRow<IT>: Sized
where
    for<'c> &'c IT: IntoIterator<Item = &'c Self>,
    IT: FromIterator<Self> + Clone,
{
    const FIELD_NUM: usize;
    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize_columns<'de, D>(de: D) -> Result<IT, D::Error>
    where
        D: serde::Deserializer<'de>;
}

pub trait MapRow<'de, K, IT>: Sized
where
    for<'c> &'c IT: IntoIterator<Item = (&'c K, &'c Self)>,
    IT: FromIterator<(K, Self)> + Clone,
    K: Serialize + Deserialize<'de> + Clone + Eq,
{
    const FIELD_NUM: usize;
    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
    where
        D: serde::Deserializer<'de>;
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ColumnarVec<'c, T: Clone>(pub Cow<'c, Vec<T>>);

impl<'c, T> ColumnarVec<'c, T>
where
    T: Clone,
{
    pub fn new(vec: Vec<T>) -> Self {
        Self(Cow::Owned(vec))
    }

    pub fn from_borrowed(vec: &'c Vec<T>) -> Self {
        Self(Cow::Borrowed(vec))
    }
}

impl<'c, T> From<ColumnarVec<'c, T>> for Vec<T>
where
    T: VecRow<Vec<T>> + Clone,
{
    fn from(vec: ColumnarVec<'c, T>) -> Self {
        vec.0.into_owned()
    }
}

impl<'c, T> Serialize for ColumnarVec<'c, T>
where
    T: VecRow<Vec<T>> + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize_columns(&self.0, serializer)
    }
}

impl<'de, 'c, T> Deserialize<'de> for ColumnarVec<'c, T>
where
    T: VecRow<Vec<T>> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ColumnarVec(Cow::Owned(T::deserialize_columns(
            deserializer,
        )?)))
    }
}
