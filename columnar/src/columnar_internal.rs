use std::ops::{Deref, DerefMut};

use postcard::{de_flavors::Flavor as DeFlavor, ser_flavors::Flavor, Deserializer, Serializer};

use crate::ColumnarError;

#[derive(Debug)]
pub struct Cursor<'de> {
    original: &'de [u8],
    pos: usize,
    end: usize,
}

impl<'de> Cursor<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
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

/// The decoder of columnar system
pub(crate) struct ColumnarDecoder<'de> {
    de: Deserializer<'de, Cursor<'de>>,
}

impl<'de> ColumnarDecoder<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        let cursor = Cursor::new(bytes);
        Self {
            de: Deserializer::from_flavor(cursor),
        }
    }

    pub fn finalize(self) -> Result<&'de [u8], ColumnarError> {
        Ok(self.de.finalize()?)
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

#[derive(Debug, Default, Clone)]
pub struct AllocVec {
    vec: Vec<u8>,
}

impl AllocVec {
    /// Create a new, currently empty, [alloc::vec::Vec] to be used for storing serialized
    /// output data.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Flavor for AllocVec {
    type Output = Vec<u8>;

    #[inline(always)]
    fn try_extend(&mut self, data: &[u8]) -> postcard::Result<()> {
        self.vec.extend_from_slice(data);
        Ok(())
    }

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        self.vec.push(data);
        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.vec)
    }
}

/// The encoder of columnar system
pub(crate) struct ColumnarEncoder {
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
