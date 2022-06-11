mod parsers;

pub use crate::parsers::{
    amounts::Amount, commodities::Commodity, descriptions::Description, journal::Journal,
    postings::Posting, prices::Price, tags::Tag, transactions::Transaction,
};

use nom::error::{ErrorKind, FromExternalError, ParseError};
use parsers::journal::Value;
use thiserror::Error;

pub type HLParserIResult<I, O> = nom::IResult<I, O, HLParserError>;
pub type HLParserResult<O> = Result<O, HLParserError>;

#[derive(Error, Debug)]
pub enum HLParserError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Parse error: {1:?} {0}")]
    Parse(String, ErrorKind),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Invalid include path: {0}")]
    IncludePath(String),
    #[error("Error extracting parsed value")]
    Extract(Value),
}

impl<'a> From<nom::error::Error<&'a str>> for HLParserError {
    fn from(err: nom::error::Error<&'a str>) -> Self {
        HLParserError::Parse(err.input.to_string(), err.code)
    }
}

impl<E> FromExternalError<&str, E> for HLParserError {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: &str, kind: ErrorKind, _e: E) -> Self {
        HLParserError::Parse(input.to_string(), kind)
    }
}

impl ParseError<&str> for HLParserError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        HLParserError::Parse(input.to_string(), kind)
    }

    fn append(_: &str, _: ErrorKind, other: Self) -> Self {
        other
    }
}
