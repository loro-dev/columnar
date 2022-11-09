use std::{borrow::Cow, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{MapRow, VecRow};

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ColumnarVec<'c, T, IT>(pub Cow<'c, IT>)
where
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
    IT: FromIterator<T> + Clone,
    T: VecRow<IT>;

impl<'c, T, IT> ColumnarVec<'c, T, IT>
where
    T: VecRow<IT>,
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
    T: VecRow<IT>,
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    fn from(vec: IT) -> Self {
        Self(Cow::Owned(vec))
    }
}

impl<'c, T, IT> From<&'c IT> for ColumnarVec<'c, T, IT>
where
    T: VecRow<IT>,
    IT: FromIterator<T> + Clone,
    for<'a> &'a IT: IntoIterator<Item = &'a T>,
{
    fn from(vec: &'c IT) -> Self {
        Self(Cow::Borrowed(vec))
    }
}

impl<'c, T, IT> Serialize for ColumnarVec<'c, T, IT>
where
    T: VecRow<IT> + Clone,
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
    T: VecRow<IT> + Clone,
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

// MapRow

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ColumnarMap<'de, 'c, K, T, IT>(pub Cow<'c, IT>, PhantomData<&'de ()>)
where
    T: MapRow<'de, K, IT> + Clone,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Serialize + Deserialize<'de> + Clone + Eq;

impl<'de, 'c, K, T, IT> ColumnarMap<'de, 'c, K, T, IT>
where
    T: MapRow<'de, K, IT> + Clone,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Serialize + Deserialize<'de> + Clone + Eq,
{
    pub fn new(map: &'c IT) -> Self {
        Self(Cow::Borrowed(map), PhantomData)
    }

    pub fn into_map(self) -> IT {
        self.0.into_owned()
    }
}

impl<'de, 'c, K, T, IT> Serialize for ColumnarMap<'de, 'c, K, T, IT>
where
    T: MapRow<'de, K, IT> + Clone,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Serialize + Deserialize<'de> + Clone + Eq,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize_columns(&self.0, serializer)
    }
}

impl<'de, 'c, K, T, IT> Deserialize<'de> for ColumnarMap<'de, 'c, K, T, IT>
where
    T: MapRow<'de, K, IT> + Clone,
    for<'a> &'a IT: IntoIterator<Item = (&'a K, &'a T)>,
    IT: FromIterator<(K, T)> + Clone,
    K: Serialize + Deserialize<'de> + Clone + Eq,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ColumnarMap(
            Cow::Owned(T::deserialize_columns(deserializer)?),
            PhantomData,
        ))
    }
}
