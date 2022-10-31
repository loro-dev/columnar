use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use serde::{de, ser};

#[derive(Debug)]
pub enum ColumnarError {
    AlreadyEnd,
    InvalidDataType,
    LengthUnknown,
    InvalidStrategy,
    DeserializeBadLeb,
    DeserializeBadChar,
    DeserializeBadUtf8,
    DeserializeBadOption,
    RleError(String),
    SerdeError(String),
}

impl Display for ColumnarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: implement this
        write!(f, "ColumnarError")
    }
}

impl Error for ColumnarError {}

impl ser::Error for ColumnarError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::SerdeError(msg.to_string())
    }
}

impl de::Error for ColumnarError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::SerdeError(msg.to_string())
    }
}
