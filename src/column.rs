use std::{marker::PhantomData, ops::DerefMut};

use serde::{Deserialize, Serialize, Serializer};

use crate::{
    columnar::ColumnarEncoder,
    strategy::{AnyRleDecoder, AnyRleEncoder, DeRle, Rle, Strategy},
    ColumnarDecoder, ColumnarError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnAttr {
    pub index: usize,
    pub strategy: Option<Strategy>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column<T: Clone> {
    pub data: Vec<T>,
    pub(crate) attr: ColumnAttr,
}

impl<T: Clone> Column<T> {
    pub fn new(data: Vec<T>, attr: ColumnAttr) -> Self {
        Self { data, attr }
    }
}

pub(crate) struct ColumnEncoder<T: Clone> {
    ser: ColumnarEncoder,
    _c: PhantomData<Column<T>>,
}
impl<'c, T> ColumnEncoder<T>
where
    T: Clone + Serialize + PartialEq,
{
    pub(crate) fn new() -> Self {
        Self {
            ser: ColumnarEncoder::new(),
            _c: PhantomData,
        }
    }

    pub(crate) fn encode(mut self, column: &Column<T>) -> Result<Vec<u8>, ColumnarError> {
        self.serialize_strategy(&column.attr.strategy)?;
        match column.attr.strategy {
            Some(Strategy::Rle) => self.encode_rle(&column.data)?,
            Some(Strategy::BoolRle) => todo!(),
            Some(Strategy::Delta) => todo!(),
            Some(Strategy::DeltaRle) => todo!(),
            None => self.encode_no_strategy(&column.data)?,
        };
        Ok(self.ser.into_bytes())
    }

    fn serialize_strategy(&mut self, strategy: &Option<Strategy>) -> Result<(), ColumnarError> {
        if let Some(strategy) = strategy {
            let n: u8 = *strategy as u8;
            self.ser
                .serialize_u8(n)
                .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))
        } else {
            self.ser
                .serialize_u8(0)
                .map_err(|e| ColumnarError::SerializeError(e as postcard::Error))
        }
    }

    #[inline]
    fn encode_rle(&mut self, column: &Vec<T>) -> Result<(), ColumnarError> {
        let mut rle_encoder = AnyRleEncoder::<T>::new(&mut self.ser);
        for data in column.into_iter() {
            rle_encoder.append(data)?
        }
        rle_encoder.finish()?;
        Ok(())
    }

    #[inline]
    fn encode_no_strategy(&mut self, column: &Vec<T>) -> Result<(), ColumnarError> {
        column.serialize(self.ser.deref_mut())?;
        Ok(())
    }
}

pub(crate) struct ColumnDecoder<'de, T: Clone> {
    de: ColumnarDecoder<'de>,
    _c: PhantomData<Column<T>>,
    lifetime: PhantomData<&'de ()>,
}

impl<'de, T> ColumnDecoder<'de, T>
where
    T: Clone + Deserialize<'de> + PartialEq,
{
    pub(crate) fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: ColumnarDecoder::new(bytes),
            _c: PhantomData,
            lifetime: PhantomData,
        }
    }

    fn deserialize_strategy(&mut self) -> Result<Option<Strategy>, ColumnarError> {
        let n = u8::deserialize(self.de.deref_mut())?;
        if n == 0 {
            Ok(None)
        } else {
            Ok(Some(Strategy::try_from(n)?))
        }
    }

    pub(crate) fn decode(&mut self) -> Result<Column<T>, ColumnarError> {
        let strategy = self.deserialize_strategy()?;
        let vec_data = match strategy {
            Some(Strategy::Rle) => self.decode_rle(),
            Some(Strategy::BoolRle) => todo!(),
            Some(Strategy::Delta) => todo!(),
            Some(Strategy::DeltaRle) => todo!(),
            None => self.decode_no_strategy(),
        };
        Ok(Column::new(vec_data?, ColumnAttr { index: 0, strategy }))
    }

    fn decode_rle(&mut self) -> Result<Vec<T>, ColumnarError> {
        let mut rle_decoder = AnyRleDecoder::<T>::new(&mut self.de);
        rle_decoder.decode()
    }

    fn decode_no_strategy(&mut self) -> Result<Vec<T>, ColumnarError> {
        Ok(Vec::<T>::deserialize(self.de.deref_mut())?)
    }
}
