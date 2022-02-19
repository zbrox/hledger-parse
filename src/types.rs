use std::{cmp::PartialEq, fmt::Display, path::PathBuf};

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

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Description {
                payee: Some(p),
                note: Some(n),
            } => write!(f, "{} | {}", p, n),
            Description {
                payee: None,
                note: Some(n),
            } => write!(f, "{}", n),
            Description {
                payee: Some(p),
                note: None,
            } => write!(f, "{} |", p),
            Description {
                payee: None,
                note: None,
            } => write!(f, ""),
        }
    }
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

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.secondary_date {
            Some(sec_date) => write!(f, "{}={} {}", self.primary_date, sec_date, self.description),
            None => write!(f, "{} {}", self.primary_date, self.description),
        }
    }
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
    #[error("Validation error: {0}")]
    Validation(String),
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
