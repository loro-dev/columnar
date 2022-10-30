use std::{borrow::Cow, fmt::Formatter, marker::PhantomData};

use serde::{
    de::{self, Error, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
    __private::from_utf8_lossy,
};

use crate::{ColumnarError, Columns};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CellData<'c> {
    U64(u64),
    I64(i64),
    String(Cow<'c, str>),
    Bool(bool),
    Bytes(Cow<'c, [u8]>),
    Columns(Columns<'c>),
}

// TODO: borrow from

impl<'c> TryFrom<CellData<'c>> for Columns<'c> {
    type Error = ColumnarError;

    fn try_from(value: CellData<'c>) -> Result<Self, Self::Error> {
        match value {
            CellData::Columns(v) => Ok(v),
            _ => Err(ColumnarError::InvalidDataType),
        }
    }
}

macro_rules! impl_try_from_cell_data {
    ($t:ty, $variant:ident) => {
        impl TryFrom<CellData<'_>> for $t {
            type Error = ColumnarError;

            fn try_from(value: CellData<'_>) -> Result<Self, Self::Error> {
                match value {
                    CellData::$variant(v) => Ok(v),
                    _ => Err(ColumnarError::InvalidDataType),
                }
            }
        }
    };
}

impl_try_from_cell_data!(u64, U64);
impl_try_from_cell_data!(i64, I64);
impl_try_from_cell_data!(bool, Bool);

macro_rules! impl_try_from_cow_cell_data {
    ($t:ty, $variant:ident) => {
        impl TryFrom<CellData<'_>> for $t {
            type Error = ColumnarError;

            fn try_from(value: CellData<'_>) -> Result<Self, Self::Error> {
                match value {
                    CellData::$variant(v) => Ok(v.into_owned()),
                    _ => Err(ColumnarError::InvalidDataType),
                }
            }
        }
    };
}

impl_try_from_cow_cell_data!(String, String);
impl_try_from_cow_cell_data!(Vec<u8>, Bytes);

impl<'de: 'c, 'c> Deserialize<'de> for CellData<'c> {
    fn deserialize<__D>(__deserializer: __D) -> Result<Self, __D::Error>
    where
        __D: Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        enum __Field {
            __field0,
            __field1,
            __field2,
            __field3,
            __field4,
            __field5,
        }
        struct __FieldVisitor;
        impl<'de> Visitor<'de> for __FieldVisitor {
            type Value = __Field;
            fn expecting(&self, __formatter: &mut Formatter) -> std::fmt::Result {
                Formatter::write_str(__formatter, "variant identifier")
            }
            fn visit_u64<__E>(self, __value: u64) -> Result<Self::Value, __E>
            where
                __E: de::Error,
            {
                match __value {
                    0u64 => Ok(__Field::__field0),
                    1u64 => Ok(__Field::__field1),
                    2u64 => Ok(__Field::__field2),
                    3u64 => Ok(__Field::__field3),
                    4u64 => Ok(__Field::__field4),
                    5u64 => Ok(__Field::__field5),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(__value),
                        &"variant index 0 <= i < 6",
                    )),
                }
            }
            fn visit_str<__E>(self, __value: &str) -> Result<Self::Value, __E>
            where
                __E: Error,
            {
                match __value {
                    "U64" => Ok(__Field::__field0),
                    "I64" => Ok(__Field::__field1),
                    "String" => Ok(__Field::__field2),
                    "Bool" => Ok(__Field::__field3),
                    "Bytes" => Ok(__Field::__field4),
                    "Columns" => Ok(__Field::__field5),
                    _ => Err(de::Error::unknown_variant(__value, VARIANTS)),
                }
            }
            fn visit_bytes<__E>(self, __value: &[u8]) -> Result<Self::Value, __E>
            where
                __E: de::Error,
            {
                match __value {
                    b"U64" => Ok(__Field::__field0),
                    b"I64" => Ok(__Field::__field1),
                    b"String" => Ok(__Field::__field2),
                    b"Bool" => Ok(__Field::__field3),
                    b"Bytes" => Ok(__Field::__field4),
                    b"Columns" => Ok(__Field::__field5),
                    _ => {
                        let __value = &from_utf8_lossy(__value);
                        Err(de::Error::unknown_variant(__value, VARIANTS))
                    }
                }
            }
        }
        impl<'de> Deserialize<'de> for __Field {
            #[inline]
            fn deserialize<__D>(__deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
            }
        }
        struct __Visitor<'de, 'c> {
            marker: PhantomData<CellData<'c>>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de: 'c, 'c> Visitor<'de> for __Visitor<'de, 'c> {
            type Value = CellData<'c>;
            fn expecting(&self, __formatter: &mut Formatter) -> std::fmt::Result {
                Formatter::write_str(__formatter, "enum CellData")
            }
            fn visit_enum<__A>(self, __data: __A) -> Result<Self::Value, __A::Error>
            where
                __A: de::EnumAccess<'de>,
            {
                match match de::EnumAccess::variant(__data) {
                    Ok(__val) => __val,
                    Err(__err) => {
                        return Err(__err);
                    }
                } {
                    (__Field::__field0, __variant) => Result::map(
                        de::VariantAccess::newtype_variant::<u64>(__variant),
                        CellData::U64,
                    ),
                    (__Field::__field1, __variant) => Result::map(
                        de::VariantAccess::newtype_variant::<i64>(__variant),
                        CellData::I64,
                    ),
                    (__Field::__field2, __variant) => Result::map(
                        de::VariantAccess::newtype_variant::<Cow<'c, str>>(__variant),
                        CellData::String,
                    ),
                    (__Field::__field3, __variant) => Result::map(
                        de::VariantAccess::newtype_variant::<bool>(__variant),
                        CellData::Bool,
                    ),
                    (__Field::__field4, __variant) => Result::map(
                        de::VariantAccess::newtype_variant::<Cow<'c, [u8]>>(__variant),
                        CellData::Bytes,
                    ),
                    (__Field::__field5, __variant) => Result::map(
                        de::VariantAccess::newtype_variant::<Columns<'c>>(__variant),
                        CellData::Columns,
                    ),
                }
            }
        }
        const VARIANTS: &'static [&'static str] =
            &["U64", "I64", "String", "Bool", "Bytes", "Columns"];
        Deserializer::deserialize_enum(
            __deserializer,
            "CellData",
            VARIANTS,
            __Visitor {
                marker: PhantomData::<CellData<'c>>,
                lifetime: PhantomData,
            },
        )
    }
}
