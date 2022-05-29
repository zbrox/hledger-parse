use std::{cmp::PartialEq, fmt::Display, path::PathBuf};

use chrono::NaiveDate;
use nom::error::{ErrorKind, FromExternalError, ParseError};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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

#[derive(PartialEq)]
pub enum AmountSign {
    Plus,
    Minus,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Amount {
    pub currency: String,
    pub value: Decimal,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Posting {
    pub status: Status,
    pub account_name: String,
    pub amount: Option<Amount>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Price {
    pub commodity: String,
    pub date: NaiveDate,
    pub amount: Amount,
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

impl<'a> Transaction {
    pub fn validate(&self) -> Result<(), HLParserError> {
        self.validate_postings()?;
        Ok(())
    }

    fn validate_postings(&self) -> Result<(), HLParserError> {
        let none_amounts = self.postings.iter().filter(|p| p.amount.is_none()).count();

        if none_amounts > 1_usize {
            return Err(HLParserError::Validation(format!(
                "Transaction {} cannot have more than 1 posting with missing amounts",
                self
            )));
        }

        if none_amounts == 1_usize {
            return Ok(());
        }

        let postings_sum = self
            .postings
            .iter()
            .flat_map(|p| &p.amount)
            .map(|a| a.value) // TODO: different currencies, conversion rates
            .sum::<Decimal>();

        if postings_sum != dec!(0) {
            return Err(HLParserError::Validation(format!(
                "Transaction {} postings' sum does not equal 0",
                self
            )));
        }

        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub struct Journal {
    pub transactions: Vec<Transaction>,
}

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
}

impl TryFrom<PathBuf> for Journal {
    type Error = HLParserError;

    fn try_from(journal_path: PathBuf) -> Result<Self, HLParserError> {
        let base_path = journal_path.parent().map(|v| v.to_owned());
        let journal_contents = std::fs::read_to_string(journal_path)?;
        let journal = parse_journal(&journal_contents, base_path).map_err(|e| match e {
            HLParserError::Parse(i, ek) => HLParserError::Parse(i, ek),
            HLParserError::Validation(desc) => HLParserError::Validation(desc),
            HLParserError::IO(e) => HLParserError::IO(e),
            HLParserError::IncludePath(path) => HLParserError::IncludePath(path),
        })?;

        Ok(journal)
    }
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

#[derive(PartialEq, Debug, Clone)]
pub struct Commodity {
    pub name: String,
    pub format: Option<String>, // TODO: temp before I decide how to store the format properly
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use super::{Amount, Description, Posting, Status, Transaction};

    #[test]
    fn test_transaction_validate_none_amount_postings() {
        let transaction = Transaction {
            primary_date: NaiveDate::from_ymd(2008, 1, 1),
            secondary_date: None,
            status: Status::Unmarked,
            code: None,
            description: Description {
                note: Some("income".into()),
                payee: None,
            },
            postings: vec![
                Posting {
                    account_name: "assets:bank:checking".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    status: Status::Unmarked,
                },
                Posting {
                    account_name: "income:salary".into(),
                    amount: None,
                    status: Status::Unmarked,
                },
            ],
            tags: vec![],
        };

        assert!(transaction.validate().is_ok());
    }

    #[test]
    fn test_transaction_validate_not_zero_sum_postings() {
        let transaction = Transaction {
            primary_date: NaiveDate::from_ymd(2008, 1, 1),
            secondary_date: None,
            status: Status::Unmarked,
            code: None,
            description: Description {
                note: Some("income".into()),
                payee: None,
            },
            postings: vec![
                Posting {
                    account_name: "assets:bank:checking".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    status: Status::Unmarked,
                },
                Posting {
                    account_name: "income:salary".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(0),
                    }),
                    status: Status::Unmarked,
                },
            ],
            tags: vec![],
        };

        assert!(transaction.validate().is_err());
    }
}
