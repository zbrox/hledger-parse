use std::{cmp::PartialEq, error::Error, fmt::Display, path::PathBuf};

use chrono::NaiveDate;

use crate::journal::parse_journal;

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

#[derive(Debug)]
pub enum ParserError {
    IO(std::io::Error),
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

impl Error for ParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ParserError::IO(ref e) => Some(e),
            ParserError::Parse(ref e) => e.source(),
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ParserError::IO(..) => write!(f, "Problem reading the journal file"),
            ParserError::Parse(..) => write!(f, "Parsing error"),
        }
    }
}

impl From<std::io::Error> for ParserError {
    fn from(err: std::io::Error) -> Self {
        ParserError::IO(err)
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
