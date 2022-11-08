use postcard::Error as PostcardError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColumnarError {
    #[error("serialize or deserialize error")]
    SerializeError(#[from] PostcardError),
    #[error("`{0}` during columnar encoding")]
    ColumnarEncodeError(String),
    #[error("`{0}` during rle encoding")]
    RleEncodeError(String),
    #[error("`{0}` during rle decoding")]
    RleDecodeError(String),
    #[error("invalid strategy code `{0}`")]
    InvalidStrategy(u8),
    #[error("unknown data store error")]
    Unknown,
}

impl serde::ser::Error for ColumnarError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        ColumnarError::ColumnarEncodeError(msg.to_string())
    }
}
