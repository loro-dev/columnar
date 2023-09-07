//!

use std::marker::PhantomData;

use crate::{
    columnar_internal::Cursor, strategy::MAX_RLE_COUNT, ColumnarError, DeltaRleable, Rleable,
};
use postcard::Deserializer;
use serde::Deserialize;

pub struct GenericIter<'de, T> {
    de: Deserializer<'de, Cursor<'de>>,
    _ty: PhantomData<T>,
}

impl<'de, T> GenericIter<'de, T>
where
    T: for<'d> Deserialize<'d>,
{
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: Deserializer::from_flavor(Cursor::new(bytes)),
            _ty: Default::default(),
        }
    }
}

impl<'de, T> Iterator for GenericIter<'de, T>
where
    T: for<'d> Deserialize<'d>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        T::deserialize(&mut self.de).ok()
    }
}

pub struct AnyRleIter<'de, T> {
    de: Deserializer<'de, Cursor<'de>>,
    last_value: Option<T>,
    count: isize,
    literal: bool,
}

impl<'de, T: Rleable> AnyRleIter<'de, T> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: Deserializer::from_flavor(Cursor::new(bytes)),
            last_value: None,
            count: 0,
            literal: false,
        }
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<T>, ColumnarError> {
        while self.count == 0 {
            let count = isize::deserialize(&mut self.de);
            if count.is_err() {
                return Ok(None);
            }
            let count = count.unwrap();
            // Prevent bad data from causing oom loops
            if count.unsigned_abs() > MAX_RLE_COUNT {
                return Err(ColumnarError::RleDecodeError(format!(
                    "decode Rle count is too large : {}",
                    self.count
                )));
            }
            match count {
                n if n > 0 => {
                    self.count = n;
                    self.last_value = Some(T::deserialize(&mut self.de)?);
                    self.literal = false;
                }
                n if n < 0 => {
                    self.count = n.abs();
                    self.literal = true;
                }
                _ => return Err(ColumnarError::RleDecodeError("Invalid count".to_string())),
            }
        }
        self.count -= 1;
        if self.literal {
            Ok(Some(T::deserialize(&mut self.de)?))
        } else {
            Ok(self.last_value.clone())
        }
    }
}

impl<'de, T: Rleable> Iterator for AnyRleIter<'de, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}

pub struct DeltaRleIter<'de, T> {
    rle_iter: AnyRleIter<'de, i128>,
    absolute_value: i128,
    _type: PhantomData<T>,
}

impl<'de, T: DeltaRleable> DeltaRleIter<'de, T> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            rle_iter: AnyRleIter::new(bytes),
            absolute_value: 0,
            _type: PhantomData,
        }
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<T>, ColumnarError> {
        let next = self.rle_iter.try_next()?;
        if let Some(delta) = next {
            self.absolute_value = self.absolute_value.saturating_add(delta);
            Ok(Some(self.absolute_value.try_into().map_err(|_| {
                ColumnarError::RleDecodeError(format!(
                    "{} cannot be safely converted from i128",
                    self.absolute_value
                ))
            })?))
        } else {
            Ok(None)
        }
    }
}

impl<'de, T: DeltaRleable> Iterator for DeltaRleIter<'de, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}

pub struct BoolRleIter<'de> {
    de: Deserializer<'de, Cursor<'de>>,
    last_value: bool,
    count: usize,
}

impl<'de> BoolRleIter<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Self {
            de: Deserializer::from_flavor(Cursor::new(bytes)),
            last_value: true,
            count: 0,
        }
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<bool>, ColumnarError> {
        while self.count == 0 {
            let count = usize::deserialize(&mut self.de);
            if count.is_err() && self.count == 0 {
                return Ok(None);
            }
            self.count = count.unwrap();
            // Prevent bad data from causing oom loops
            if self.count > MAX_RLE_COUNT {
                return Err(ColumnarError::RleDecodeError(format!(
                    "decode Rle count is too large : {}",
                    self.count
                )));
            }
            self.last_value = !self.last_value;
        }
        self.count -= 1;
        Ok(Some(self.last_value))
    }
}

impl<'de> Iterator for BoolRleIter<'de> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}

pub trait TableIter<'de> {
    type Iter: Deserialize<'de>;
}

#[macro_export]
macro_rules! multi_zip {
    ($first:expr $(, $rest:expr)*) => {
        {
            let mut _zipped = $first;
            $(
                _zipped = _zipped.zip($rest);
            )*

            _zipped.map(
                |nested_tuple| {
                    flatten_tuple!(nested_tuple)
                }
            )
        }
    };
}

#[macro_export]
macro_rules! flatten_tuple {
    (($first:expr, $second:expr)) => {
        ($first, $second)
    };

    (($first:expr, $rest:tt)) => {
        {
            let ($first_value, nested_tuple) = ($first, $rest);
            let flattened_rest = flatten_tuple!(nested_tuple);
            ($first_value, $(flattened_rest).*)
        }
    };
}

// TODO: compress
impl<'de, T: Rleable> Deserialize<'de> for AnyRleIter<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut bytes: &'de [u8] = Deserialize::deserialize(deserializer)?;
        #[cfg(feature = "compress")]
        {}
        #[cfg(not(feature = "compress"))]
        {
            bytes = &bytes[1..]
        }
        Ok(AnyRleIter::new(bytes))
    }
}

impl<'de, T: DeltaRleable> Deserialize<'de> for DeltaRleIter<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut bytes: &'de [u8] = Deserialize::deserialize(deserializer)?;
        #[cfg(feature = "compress")]
        {}
        #[cfg(not(feature = "compress"))]
        {
            bytes = &bytes[1..]
        }
        Ok(DeltaRleIter::new(bytes))
    }
}

impl<'de> Deserialize<'de> for BoolRleIter<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut bytes: &'de [u8] = Deserialize::deserialize(deserializer)?;
        #[cfg(feature = "compress")]
        {}
        #[cfg(not(feature = "compress"))]
        {
            bytes = &bytes[1..]
        }
        Ok(BoolRleIter::new(bytes))
    }
}

impl<'de, T> Deserialize<'de> for GenericIter<'de, T>
where
    T: for<'d> Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut bytes: &'de [u8] = Deserialize::deserialize(deserializer)?;
        println!("de {:?}", bytes);
        #[cfg(feature = "compress")]
        {}
        #[cfg(not(feature = "compress"))]
        {
            bytes = &bytes[1..]
        }
        Ok(GenericIter::new(bytes))
    }
}
