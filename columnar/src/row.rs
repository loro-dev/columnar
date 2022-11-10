use serde::{Deserialize, Serialize};

/// If a type implements [`RowSer`] and [`RowDe`] trait, it can be considered as a row of vec-like container.
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
/// so we can implement the [`RowSer`] and [`RowDe`] trait for `Data`, they will provide two functions
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
/// [`RowSer`] trait has a generic type `IT`, which could be any container that can be IntoIterator<Item = &Self>
/// and FromIterator<Self>, such as Vec<T>, SmallVec<T> and so on.
///
/// [`serialize_columns()`]: RowSer::serialize_columns()
/// [`deserialize_columns()`]: RowDe::deserialize_columns()
/// [`Column`]: crate::column::Column
/// [`DeltaRle`]: crate::strategy::DeltaRleEncoder
/// [`ColumnEncoder`]: crate::column::ColumnEncoder
pub trait RowSer<IT>: Sized + Serialize
where
    for<'c> &'c IT: IntoIterator<Item = &'c Self>,
{
    const FIELD_NUM: usize;
    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

pub trait RowDe<'de, IT>: Sized + Deserialize<'de>
where
    IT: FromIterator<Self> + Clone,
{
    const FIELD_NUM: usize;

    fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
    where
        D: serde::Deserializer<'de>;
}

/// The **HashMap** version of [`KeyRowSer`] trait.
///
/// Almost the same as [`KeyRowSer`], but additionally needs to convert arbitrary type K to Vec<K>.
///
pub trait KeyRowSer<K, IT>: Sized + Serialize
where
    for<'c> &'c IT: IntoIterator<Item = (&'c K, &'c Self)>,
    K: Serialize + Eq + Clone,
{
    const FIELD_NUM: usize;
    fn serialize_columns<S>(rows: &IT, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

pub trait KeyRowDe<'de, K, IT>: Sized + Deserialize<'de>
where
    IT: FromIterator<(K, Self)> + Clone,
    K: Deserialize<'de> + Eq + Clone,
{
    const FIELD_NUM: usize;

    fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
    where
        D: serde::Deserializer<'de>;
}
