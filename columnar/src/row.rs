use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// If a type implements this [`VecRow`] trait, it can be considered as a row of vec-like container.
///
/// this trait can **be easily derived** by adding `#[columnar(vec)]` to the struct.
///
/// For example, there is a struct `Data` has two fields `id` and `name`.
///
/// ```rust
/// #[columnar(vec)]
/// struct Data {
///     #[columnar(index = 1, strategy = "DeltaRle")]
///     id: u32,
///     name: String,
/// }
/// ```
/// If put `Data` into a vec-like container,
/// such as `Vec<Data>`, all the data will be showed as like:
///
/// |  id   | name  |
/// |  ----  | ----  |
/// | 1  | "Alice" |
/// | 2  | "Bob" |
/// | 3  | "Mark" |
///
/// In the columnar system, we want to store the data by [Column-oriented Storage](https://en.wikipedia.org/wiki/Column-oriented_DBMS),
/// so we can implement the [`VecRow`] trait for `Data`, and [`VecRow`] trait will provide two functions
///
/// -  [`serialize_columns()`]
/// -  [`deserialize_columns()`]
///
/// to convert the data into [`Column`]s and convert the [`Column`]s back to the original data.
///
/// For example there is a container such as:
///
/// ```rust ignore
/// #[columnar]
/// struct Store{
///     #[columnar(type="vec")]
///     data: Vec<Data>,
/// }
/// ```
/// when we serialize the `Store`, `data` field will be converted into two columns:
///
/// - ids: Vec<id> = vec![1, 2, 3] with Strategy::DeltaRle
/// - names: Vec<name> = vec!["Alice", "Bob", "Mark"] without Strategy
///
/// for `ids`, as [`Column`] type, they can be compressed by [`DeltaRle`] easily in [`ColumnEncoder`] system.
///
/// # Note:
///
/// [`VecRow`] trait has a generic type `IT`, which could be any container that can be IntoIterator<Item = &Self>
/// and FromIterator<Self>, such as Vec<T>, SmallVec<T> and so on.
///
/// [`serialize_columns()`]: VecRow::serialize_columns()
/// [`deserialize_columns()`]: VecRow::deserialize_columns()
/// [`Column`]: crate::column::Column
/// [`DeltaRle`]: crate::strategy::DeltaRleEncoder
/// [`ColumnEncoder`]: crate::column::ColumnEncoder
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

/// The **HashMap** version of [`VecRow`] trait.
///
/// Almost the same as [`VecRow`], but additionally needs to handle arbitrary type K to Vec<K>.
///
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
