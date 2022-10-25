use std::{error::Error, fmt::Display};


#[derive(Debug)]
pub enum EncodeError {
    NotImplemented,
}
impl Error for EncodeError {}
impl Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not implemented")
    }
}