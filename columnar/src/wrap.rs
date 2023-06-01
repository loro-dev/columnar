use std::{borrow::Cow, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::row::{KeyRowDe, KeyRowSer, RowDe, RowSer};

// TODO: remove clone

/// The wrapper of `Vec-like` container, we have implemented the `Serialize` and `Deserialize` for it.
///
/// When it is serialized or deserialized, it will call [`RowSer::serialize_columns()`] or [`RowDe::deserialize_columns()`]
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ColumnarVec<'c, T, IT>(pub Cow<'c, IT>)
where
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
    IT: FromIterator<T> + Clone;

impl<'c, T, IT> ColumnarVec<'c, T, IT>
where
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    pub fn new(vec: &'c IT) -> Self {
        Self(Cow::Borrowed(vec))
    }

    pub fn into_vec(self) -> IT {
        self.0.into_owned()
    }
}

impl<T, IT> From<IT> for ColumnarVec<'_, T, IT>
where
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    fn from(vec: IT) -> Self {
        Self(Cow::Owned(vec))
    }
}

impl<'c, T, IT> From<&'c IT> for ColumnarVec<'c, T, IT>
where
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    fn from(vec: &'c IT) -> Self {
        Self(Cow::Borrowed(vec))
    }
}

impl<'c, T, IT> Serialize for ColumnarVec<'c, T, IT>
where
    T: RowSer<IT>,
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize_columns(&self.0, serializer)
    }
}

impl<'de, 'c, T, IT> Deserialize<'de> for ColumnarVec<'c, T, IT>
where
    T: RowDe<'de, IT>,
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
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

/// The wrapper of `Map-like` container, we have implemented the `Serialize` and `Deserialize` for it.
///
/// When it is serialized or deserialized, it will call [`KeyRowSer::serialize_columns()`] or [`KeyRowDe::deserialize_columns()`]
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ColumnarMap<'c, K, T, IT>(pub Cow<'c, IT>)
where
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Eq + Clone;

impl<'c, K, T, IT> ColumnarMap<'c, K, T, IT>
where
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Eq + Clone,
{
    pub fn new(map: &'c IT) -> Self {
        Self(Cow::Borrowed(map))
    }

    pub fn into_map(self) -> IT {
        self.0.into_owned()
    }
}

impl<'c, K, T, IT> Serialize for ColumnarMap<'c, K, T, IT>
where
    T: KeyRowSer<K, IT>,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Serialize + PartialEq + Eq + Hash + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize_columns(&self.0, serializer)
    }
}

impl<'de, 'c, K, T, IT> Deserialize<'de> for ColumnarMap<'c, K, T, IT>
where
    T: KeyRowDe<'de, K, IT>,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Deserialize<'de> + PartialEq + Eq + Hash + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ColumnarMap(Cow::Owned(T::deserialize_columns(
            deserializer,
        )?)))
    }
}

impl<'c, T, IT> Default for ColumnarVec<'c, T, IT>
where
    IT: FromIterator<T> + Clone + Default,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'de, 'c, K, T, IT> Default for ColumnarMap<'c, K, T, IT>
where
    T: KeyRowDe<'de, K, IT>,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone + Default,
    K: Deserialize<'de> + PartialEq + Eq + Hash + Clone,
{
    fn default() -> Self {
        Self(Default::default())
    }
}
