use std::{fmt::{Display, Formatter, Debug}, error::Error};

use serde::{ser, de};


#[derive(Debug)]
pub enum ColumnarError{
    AlreadyEnd,
    InvalidDataType,
    LengthUnknown,
    SerdeError(String)
}

impl Display for ColumnarError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ColumnarError")
    }
    
}

impl Error for ColumnarError{}

impl ser::Error for ColumnarError{
    fn custom<T: Display>(msg: T) -> Self {
        Self::SerdeError(msg.to_string())
    }
}

impl de::Error for ColumnarError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::SerdeError(msg.to_string())
    }
}