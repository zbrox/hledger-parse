use std::{cmp::PartialEq, path::PathBuf};

use chrono::NaiveDate;
use thiserror::Error;

use crate::parsers::journal::parse_journal;

#[derive(PartialEq, Debug, Clone)]
pub enum Status {
    Unmarked,
    Pending,
    Cleared,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Description {
    pub payee: Option<String>,
    pub note: Option<String>,
}

pub enum AmountSign {
    Plus,
    Minus,
}

impl AmountSign {
    pub fn multiplier(&self) -> i8 {
        match *self {
            AmountSign::Plus => 1,
            AmountSign::Minus => -1,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Amount {
    pub currency: String,
    pub value: i32,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Posting {
    pub status: Status,
    pub account_name: String,
    pub amount: Option<Amount>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub value: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Transaction {
    pub primary_date: NaiveDate,
    pub secondary_date: Option<NaiveDate>,
    pub status: Status,
    pub code: Option<String>,
    pub description: Description,
    pub postings: Vec<Posting>,
    pub tags: Vec<Tag>,
}

#[derive(PartialEq, Debug)]
pub struct Journal {
    pub transactions: Vec<Transaction>,
}

pub type ParserResult<T> = std::result::Result<T, ParserError>;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(nom::error::Error<String>),
}

impl TryFrom<PathBuf> for Journal {
    type Error = ParserError;

    fn try_from(journal_path: PathBuf) -> ParserResult<Self> {
        let journal_contents = std::fs::read_to_string(journal_path)?;
        let journal = parse_journal(&journal_contents)?;

        Ok(journal)
    }
}

impl From<nom::error::Error<&str>> for ParserError {
    fn from(err: nom::error::Error<&str>) -> Self {
        ParserError::Parse(nom::error::Error::new(err.input.to_string(), err.code))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::AmountSign;

    #[test]
    fn test_amount_sign_multiplier() {
        assert_eq!(AmountSign::Plus.multiplier(), 1);
        assert_eq!(AmountSign::Minus.multiplier(), -1);
    }
}
