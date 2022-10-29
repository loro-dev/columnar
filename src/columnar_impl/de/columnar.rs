use serde::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
};
use serde::Deserializer;

use crate::{
    columnar_impl::leb128::{max_of_last_byte, varint_max},
    ColumnarError,
};

use super::cursor::Cursor;

pub struct ColumnarDeserializer<'de> {
    cursor: Cursor<'de>,
}

impl<'de> ColumnarDeserializer<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            cursor: Cursor::new(bytes),
        }
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut ColumnarDeserializer<'de> {
    type Error = ColumnarError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(ColumnarError::SerdeError(
            "Do not support deserialize_any".to_string(),
        ))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = match self.cursor.pop()? {
            0 => false,
            1 => true,
            _ => {
                return Err(ColumnarError::SerdeError(
                    "Deserialize bool error".to_string(),
                ))
            }
        };
        visitor.visit_bool(v)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.cursor.pop()? as i8)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = self.take_leb_u16()?;
        visitor.visit_u16(v)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = self.take_leb_u32()?;
        visitor.visit_u32(v)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = self.take_leb_u64()?;
        visitor.visit_u64(v)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.cursor.pop()?)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = self.take_leb_u16()?;
        visitor.visit_u16(v)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = self.take_leb_u32()?;
        visitor.visit_u32(v)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let v = self.take_leb_u64()?;
        visitor.visit_u64(v)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let bytes = self.cursor.try_take_n(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        visitor.visit_f32(f32::from_bits(u32::from_le_bytes(buf)))
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let bytes = self.cursor.try_take_n(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        visitor.visit_f64(f64::from_bits(u64::from_le_bytes(buf)))
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let sz = self.take_leb_usize()?;
        if sz > 4 {
            return Err(ColumnarError::DeserializeBadChar);
        }
        let bytes: &'de [u8] = self.cursor.try_take_n(sz)?;
        // we pass the character through string conversion because
        // this handles transforming the array of code units to a
        // codepoint. we can't use char::from_u32() because it expects
        // an already-processed codepoint.
        let character = core::str::from_utf8(bytes)
            .map_err(|_| ColumnarError::DeserializeBadChar)?
            .chars()
            .next()
            .ok_or(ColumnarError::DeserializeBadChar)?;
        visitor.visit_char(character)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let sz = self.take_leb_usize()?;
        let bytes: &'de [u8] = self.cursor.try_take_n(sz)?;
        let str_sl = core::str::from_utf8(bytes).map_err(|_| ColumnarError::DeserializeBadUtf8)?;

        visitor.visit_borrowed_str(str_sl)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let sz = self.take_leb_usize()?;
        let bytes: &'de [u8] = self.cursor.try_take_n(sz)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.cursor.pop()? {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(ColumnarError::DeserializeBadOption),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len = self.take_leb_usize()?;
        visitor.visit_seq(ColumnarSeqAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ColumnarSeqAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len = self.take_leb_usize()?;
        visitor.visit_map(ColumnarMapAccess {
            deserializer: self,
            len,
        })
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(ColumnarError::SerdeError(
            "Do not support deserialize_identifier".to_string(),
        ))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(ColumnarError::SerdeError(
            "Do not support deserialize_ignored_any".to_string(),
        ))
    }
}

impl<'de> ColumnarDeserializer<'de> {
    #[inline]
    fn take_leb_usize(&mut self) -> Result<usize, ColumnarError> {
        // TODO: u32
        self.take_leb_u64().map(|u| u as usize)
    }

    #[inline]
    fn take_leb_u64(&mut self) -> Result<u64, ColumnarError> {
        let mut out = 0;
        for i in 0..varint_max::<u64>() {
            let val = self.cursor.pop()?;
            let carry = (val & 0x7F) as u64;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u64>() - 1 && val > max_of_last_byte::<u64>() {
                    return Err(ColumnarError::DeserializeBadLeb);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(ColumnarError::DeserializeBadLeb)
    }

    #[inline]
    fn take_leb_u32(&mut self) -> Result<u32, ColumnarError> {
        let mut out = 0;
        for i in 0..varint_max::<u32>() {
            let val = self.cursor.pop()?;
            let carry = (val & 0x7F) as u32;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u32>() - 1 && val > max_of_last_byte::<u32>() {
                    return Err(ColumnarError::DeserializeBadLeb);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(ColumnarError::DeserializeBadLeb)
    }

    #[inline]
    fn take_leb_u16(&mut self) -> Result<u16, ColumnarError> {
        let mut out = 0;
        for i in 0..varint_max::<u16>() {
            let val = self.cursor.pop()?;
            let carry = (val & 0x7F) as u16;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u16>() - 1 && val > max_of_last_byte::<u16>() {
                    return Err(ColumnarError::DeserializeBadLeb);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(ColumnarError::DeserializeBadLeb)
    }
}

impl<'de, 'a> VariantAccess<'de> for &'a mut ColumnarDeserializer<'de> {
    type Error = ColumnarError;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    #[inline]
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }
}

struct ColumnarSeqAccess<'a, 'b: 'a> {
    deserializer: &'a mut ColumnarDeserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> SeqAccess<'b> for ColumnarSeqAccess<'a, 'b> {
    type Error = ColumnarError;

    #[inline]
    fn next_element_seed<V: DeserializeSeed<'b>>(
        &mut self,
        seed: V,
    ) -> Result<Option<V::Value>, ColumnarError> {
        if self.len > 0 {
            self.len -= 1;
            Ok(Some(DeserializeSeed::deserialize(
                seed,
                &mut *self.deserializer,
            )?))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct ColumnarMapAccess<'a, 'b: 'a> {
    deserializer: &'a mut ColumnarDeserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> MapAccess<'b> for ColumnarMapAccess<'a, 'b> {
    type Error = ColumnarError;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'b>,
    {
        if self.len > 0 {
            self.len -= 1;
            Ok(Some(DeserializeSeed::deserialize(
                seed,
                &mut *self.deserializer,
            )?))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'b>,
    {
        DeserializeSeed::deserialize(seed, &mut *self.deserializer)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de, 'a> EnumAccess<'de> for &'a mut ColumnarDeserializer<'de> {
    type Error = ColumnarError;
    type Variant = Self;

    #[inline]
    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self), ColumnarError> {
        let varint = self.take_leb_u32()?;
        let v = DeserializeSeed::deserialize(seed, varint.into_deserializer())?;
        Ok((v, self))
    }
}
