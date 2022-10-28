use super::{
    leb128::uleb64,
    zigzag::{zig_zag_i16, zig_zag_i32, zig_zag_i64},
};
use crate::ColumnarError;
use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};

#[derive(Debug)]
pub struct ColumnarSerializer {
    buf: Vec<u8>,
}

impl ColumnarSerializer {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub(crate) fn to_bytes(self) -> Vec<u8> {
        self.buf
    }
}

impl<'a> Serializer for &'a mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(if v { 1 } else { 0 })
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(v.to_le_bytes()[0])
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        let zzv = zig_zag_i16(v);
        self.serialize_u16(zzv)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let zzv = zig_zag_i32(v);
        self.serialize_u32(zzv)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let zzv = zig_zag_i64(v);
        self.serialize_u64(zzv)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.buf.extend_from_slice(uleb64(v).as_slice());
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let buf = v.to_bits().to_le_bytes();
        self.buf.extend_from_slice(&buf);
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let buf = v.to_bits().to_le_bytes();
        self.buf.extend_from_slice(&buf);
        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        let str = v.encode_utf8(&mut buf);
        str.serialize(self)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.len() as u64)?;
        self.buf.extend_from_slice(v.as_bytes());
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.len() as u64)?;
        self.buf.extend_from_slice(v);
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(0)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(variant_index)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(l) = len {
            self.serialize_u64(l as u64)?;
        }
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_u64(variant_index as u64)?;
        Ok(self)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serialize_u64(len.ok_or(ColumnarError::LengthUnknown)? as u64)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_u64(len as u64)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _key: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_u64(variant_index as u64)?;
        Ok(self)
    }
}

impl SerializeSeq for &mut ColumnarSerializer {
    type Ok = ();
    type Error = ColumnarError;

    #[inline]
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for &mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeMap for &mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStructVariant for &mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleVariant for &mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleStruct for &mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for &mut ColumnarSerializer {
    type Ok = ();

    type Error = ColumnarError;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

mod test {
    #[test]
    fn test_columnar() {
        use std::borrow::Cow;

        use serde::Serialize;
        use serde_with::serde_as;

        use crate::{
            columnar_impl::ser::columnar::ColumnarSerializer, CellData, ColumnAttr, Columns, Row,
            Strategy,
        };

        #[derive(Debug)]
        struct Data {
            // #[columnar(strategy = "RLE")]
            id: u64,
            name: String,
            age: u32,
        }

        impl Row for Data {
            fn get_attrs() -> Vec<ColumnAttr> {
                vec![
                    ColumnAttr {
                        index: 1,
                        strategies: vec![Strategy::Rle, Strategy::Plain],
                    },
                    ColumnAttr {
                        index: 2,
                        strategies: vec![Strategy::Plain],
                    },
                    ColumnAttr {
                        index: 3,
                        strategies: vec![Strategy::Plain],
                    },
                ]
            }

            fn get_cells_data<'a: 'c, 'c>(&'a self) -> Vec<CellData<'c>> {
                vec![
                    CellData::U64(self.id),
                    CellData::String(Cow::Borrowed(&self.name)),
                    CellData::U64(self.age as u64),
                ]
            }
        }

        #[serde_as]
        #[derive(Debug, Serialize)]
        struct Store {
            #[serde_as(as = "Columns")]
            pub a: Vec<Data>,
            pub b: String,
        }

        let store = Store {
            a: vec![
                Data {
                    id: 2,
                    name: "a".to_string(),
                    age: 1,
                },
                Data {
                    id: 2,
                    name: "b".to_string(),
                    age: 2,
                },
                Data {
                    id: 2,
                    name: "c".to_string(),
                    age: 3,
                },
            ],
            b: "b".to_string(),
        };
        let mut columnar = ColumnarSerializer::new();
        store.serialize(&mut columnar).unwrap();
        println!("{:?}", columnar.to_bytes());
    }
}
