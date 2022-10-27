use serde::{Serialize, Serializer, ser::{Impossible, SerializeSeq, SerializeStruct}};
use crate::{ColumnarError, columnar_impl::ser::{low_bits_of_u64, CONTINUATION_BIT}};

#[derive(Debug)]
pub struct ColumnarSerializer{
    buf: Vec<u8>
}

impl ColumnarSerializer {
    pub fn new() -> Self{
        Self{buf: Vec::new()}
    }

    pub(crate) fn buf(&self) -> &Vec<u8>{
        &self.buf
    }

    pub(crate) fn to_bytes(self) -> Vec<u8>{
        self.buf
    }
}

impl<'a> Serializer for &'a mut ColumnarSerializer{
    type Ok= ();

    type Error = ColumnarError;

    type SerializeSeq = Self;

    type SerializeTuple = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Self;

    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.buf.push(v as u8);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let mut val = v;
        let mut _bytes_written = 0;
        loop {
            let mut byte = val as u8;
            // Keep the sign bit for testing
            val >>= 6;
            let done = val == 0 || val == -1;
            if done {
                byte &= !CONTINUATION_BIT;
            } else {
                // Remove the sign bit
                val >>= 1;
                // More bytes to come, so set the continuation bit.
                byte |= CONTINUATION_BIT;
            }

            self.buf.push(byte);
            _bytes_written += 1;

            if done {
                return Ok(())
            }
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let mut val = v;
        let mut _bytes_written = 0;
        loop {
            let mut byte = low_bits_of_u64(val);
            val >>= 7;
            if val != 0 {
                // More bytes to come, so set the continuation bit.
                byte |= CONTINUATION_BIT;
            }

            self.buf.push(byte);
            _bytes_written += 1;

            if val == 0 {
                break;
            }
        }
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.buf.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.buf.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(l) = len{
            self.serialize_u64(l as u64)?;
        }
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_u64(len as u64)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}


impl SerializeSeq for & mut ColumnarSerializer{
    type Ok = ();
    type Error = ColumnarError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for &mut ColumnarSerializer {
    type Ok=();

    type Error=ColumnarError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize {
        // TODO: key is unnecessary?
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

mod test{
    use std::borrow::Cow;

    use serde::Serialize;
    use serde_with::serde_as;

    use crate::{Row, ColumnAttr, Strategy, ColumnData, Columns, columnar_impl::ser::columnar::ColumnarSerializer};


    #[test]
    fn test_columnar(){
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

            fn get_columns_data<'a: 'c, 'c>(&'a self) -> Vec<ColumnData<'c>> {
                vec![
                    ColumnData::U64(self.id),
                    ColumnData::String(Cow::Borrowed(&self.name)),
                    ColumnData::U64(self.age as u64),
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
        store.serialize(&mut columnar);
        println!("{:?}", columnar.buf());
    }
}