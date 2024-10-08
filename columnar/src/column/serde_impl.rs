use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    column::delta_of_delta::DeltaOfDeltable, BoolRleColumn, ColumnTrait, DeltaOfDeltaColumn,
    DeltaRleColumn, DeltaRleable, GenericColumn, RleColumn, Rleable,
};

impl<T: Rleable> Serialize for RleColumn<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.encode().map_err(|e| {
            // eprintln!("Column Serialize Error: {:?}", e);
            serde::ser::Error::custom(e.to_string())
        })?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<T: DeltaOfDeltable> Serialize for DeltaOfDeltaColumn<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.encode().map_err(|e| {
            // eprintln!("Column Serialize Error: {:?}", e);
            serde::ser::Error::custom(e.to_string())
        })?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<T: DeltaRleable> Serialize for DeltaRleColumn<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.encode().map_err(|e| {
            // eprintln!("Column Serialize Error: {:?}", e);
            serde::ser::Error::custom(e.to_string())
        })?;
        serializer.serialize_bytes(&bytes)
    }
}

impl Serialize for BoolRleColumn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.encode().map_err(|e| {
            // eprintln!("Column Serialize Error: {:?}", e);
            serde::ser::Error::custom(e.to_string())
        })?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for BoolRleColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor;
        impl<'de> serde::de::Visitor<'de> for ColumnVisitor {
            type Value = BoolRleColumn;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a columnar encoded bool rle column")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                BoolRleColumn::decode(v).map_err(|e| {
                    eprintln!("Column Deserialize Error: {:?}", e);
                    serde::de::Error::custom(e.to_string())
                })
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor)
    }
}

impl<'de, T: DeltaRleable> Deserialize<'de> for DeltaRleColumn<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor<T>(PhantomData<T>);
        impl<'de, T: DeltaRleable> serde::de::Visitor<'de> for ColumnVisitor<T> {
            type Value = DeltaRleColumn<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a columnar encoded delta rle column")
            }
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                DeltaRleColumn::decode(v).map_err(|e| {
                    eprintln!("Column Deserialize Error: {:?}", e);
                    serde::de::Error::custom(e.to_string())
                })
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor(Default::default()))
    }
}

impl<'de, T: DeltaOfDeltable> Deserialize<'de> for DeltaOfDeltaColumn<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor<T>(PhantomData<T>);
        impl<'de, T: DeltaOfDeltable> serde::de::Visitor<'de> for ColumnVisitor<T> {
            type Value = DeltaOfDeltaColumn<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a columnar encoded delta of delta column")
            }
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                DeltaOfDeltaColumn::decode(v).map_err(|e| {
                    eprintln!("Column Deserialize Error: {:?}", e);
                    serde::de::Error::custom(e.to_string())
                })
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor(Default::default()))
    }
}

impl<'de, T: Rleable> Deserialize<'de> for RleColumn<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor<T>(PhantomData<T>);
        impl<'de, T> serde::de::Visitor<'de> for ColumnVisitor<T>
        where
            T: Rleable,
        {
            type Value = RleColumn<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a columnar encoded rle column")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                RleColumn::decode(v).map_err(|e| {
                    eprintln!("Column Deserialize Error: {:?}", e);
                    serde::de::Error::custom(e.to_string())
                })
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor(Default::default()))
    }
}

impl<T> Serialize for GenericColumn<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.encode().map_err(|e| {
            // eprintln!("Column Serialize Error: {:?}", e);
            serde::ser::Error::custom(e.to_string())
        })?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de, T> Deserialize<'de> for GenericColumn<T>
where
    T: Serialize + for<'d> Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ColumnVisitor<T>(PhantomData<T>);
        impl<'de, T> serde::de::Visitor<'de> for ColumnVisitor<T>
        where
            T: Serialize + for<'d> serde::Deserialize<'d>,
        {
            type Value = GenericColumn<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a columnar encoded generic column")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                GenericColumn::decode(v).map_err(|e| {
                    eprintln!("Column Deserialize Error: {:?}", e);
                    serde::de::Error::custom(e.to_string())
                })
            }
        }
        deserializer.deserialize_bytes(ColumnVisitor(Default::default()))
    }
}
