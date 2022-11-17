use std::ops::{Deref, DerefMut};

use postcard::{
    de_flavors::Flavor as DeFlavor,
    ser_flavors::{AllocVec, Flavor},
    Deserializer, Serializer,
};

pub struct Cursor<'de> {
    original: &'de [u8],
    pos: usize,
    end: usize,
}

impl<'de> Cursor<'de> {
    fn new(bytes: &'de [u8]) -> Self {
        Self {
            pos: 0,
            original: bytes,
            end: bytes.len(),
        }
    }
}

impl<'de> DeFlavor<'de> for Cursor<'de> {
    type Remainder = &'de [u8];
    type Source = &'de [u8];

    fn pop(&mut self) -> postcard::Result<u8> {
        if self.pos + 1 > self.end {
            Err(postcard::Error::DeserializeUnexpectedEnd)
        } else {
            let res = Ok(self.original[self.pos]);
            self.pos += 1;
            res
        }
    }

    fn try_take_n(&mut self, ct: usize) -> postcard::Result<&'de [u8]> {
        if self
            .pos
            .checked_add(ct)
            .ok_or(postcard::Error::DeserializeUnexpectedEnd)?
            > self.end
        {
            Err(postcard::Error::DeserializeUnexpectedEnd)
        } else {
            let sli = &self.original[self.pos..self.pos + ct];
            self.pos += ct;
            Ok(sli)
        }
    }

    fn finalize(self) -> postcard::Result<Self::Remainder> {
        Ok(&self.original[self.pos..])
    }
}

pub struct ColumnarDecoder<'de> {
    de: Deserializer<'de, Cursor<'de>>,
}

impl<'de> ColumnarDecoder<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        let cursor = Cursor::new(bytes);
        Self {
            de: Deserializer::from_flavor(cursor),
        }
    }
}

impl<'de> Deref for ColumnarDecoder<'de> {
    type Target = Deserializer<'de, Cursor<'de>>;

    fn deref(&self) -> &Self::Target {
        &self.de
    }
}

impl DerefMut for ColumnarDecoder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.de
    }
}

pub struct ColumnarEncoder {
    ser: Serializer<AllocVec>,
}

impl ColumnarEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.ser.output.finalize().unwrap()
    }
}

impl Default for ColumnarEncoder {
    fn default() -> Self {
        Self {
            ser: Serializer {
                output: AllocVec::new(),
            },
        }
    }
}

impl Deref for ColumnarEncoder {
    type Target = Serializer<AllocVec>;

    fn deref(&self) -> &Self::Target {
        &self.ser
    }
}

impl DerefMut for ColumnarEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ser
    }
}
