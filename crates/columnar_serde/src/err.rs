use std::{fmt::{Display, Formatter}, error::Error};


#[derive(Debug)]
pub enum ColumnarError{
    AlreadyEnd
}

impl Display for ColumnarError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ColumnarError")
    }
    
}

impl Error for ColumnarError{}