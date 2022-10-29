use std::ops::Range;

use crate::ColumnarError;

pub(crate) struct Cursor<'de> {
    bytes: &'de [u8],
    range: Range<usize>,
}

impl<'de> Cursor<'de> {
    pub(crate) fn new(bytes: &'de [u8]) -> Self {
        Self {
            bytes,
            range: 0..bytes.len(),
        }
    }

    pub(crate) fn pop(&mut self) -> Result<u8, ColumnarError> {
        if self.is_empty() {
            return Err(ColumnarError::AlreadyEnd);
        }
        let value = self.bytes[self.range.start];
        self.range.start += 1;
        Ok(value)
    }

    pub(crate) fn try_take_n(&mut self, n: usize) -> Result<&'de [u8], ColumnarError> {
        if n > self.remaining() {
            return Err(ColumnarError::AlreadyEnd);
        }
        let value = &self.bytes[self.range.start..self.range.start + n];
        self.range.start += n;
        Ok(value)
    }

    pub(crate) fn remaining(&self) -> usize {
        self.range.end - self.range.start
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.range.start >= self.range.end
    }

    pub(crate) fn remain(&self) -> &'de [u8] {
        &self.bytes[self.range.start..self.range.end]
    }
}
