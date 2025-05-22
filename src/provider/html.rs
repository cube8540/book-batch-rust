pub mod kyobo;

use crate::item::BookBuilder;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsingError {
    ArgumentError(String),
    AuthenticationError(String),
    PageNotFound(String),
    ElementNotFound(String),
    UnknownError(String),
    RequestFailed(String),
    ResponseTextExtractionFailed(String),
    ItemNotFound,
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait Client {
    fn get(&self, isbn: &str) -> Result<BookBuilder, ParsingError>;
}