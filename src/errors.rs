use crate::parser::ParserError;
use std::io;

#[derive(Debug)]
pub enum ReplError {
    ParserError(ParserError),
    IoError(io::Error),
    SymbolUndefined(String),
}

impl From<ParserError> for ReplError {
    fn from(value: ParserError) -> ReplError {
        ReplError::ParserError(value)
    }
}

impl From<io::Error> for ReplError {
    fn from(value: io::Error) -> ReplError {
        ReplError::IoError(value)
    }
}
