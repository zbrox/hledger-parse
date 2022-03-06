use std::{cmp::PartialEq, fmt::Display, path::PathBuf};

use chrono::NaiveDate;
use nom::error::{ErrorKind, FromExternalError, ParseError};
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

impl<'a> Transaction {
    pub fn validate<I>(&self) -> Result<(), HLParserError<I>> {
        self.validate_postings()?;
        Ok(())
    }

    fn validate_postings<I>(&self) -> Result<(), HLParserError<I>> {
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
            .map(|p| &p.amount)
            .flatten()
            .map(|a| a.value) // TODO: different currencies, conversion rates
            .sum::<i32>();

        if postings_sum != 0 {
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

pub type HLParserIResult<I, O> = nom::IResult<I, O, HLParserError<I>>;

#[derive(Error, Debug)]
pub enum HLParserError<I> {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(I, ErrorKind),
    #[error("Validation error: {0}")]
    Validation(String),
}

impl TryFrom<PathBuf> for Journal {
    type Error = HLParserError<String>;

    fn try_from(journal_path: PathBuf) -> Result<Self, HLParserError<String>> {
        let journal_contents = std::fs::read_to_string(journal_path)?;
        let journal = parse_journal(&journal_contents).map_err(|e| match e {
            HLParserError::Parse(i, ek) => HLParserError::Parse(i.to_string(), ek),
            HLParserError::Validation(desc) => HLParserError::Validation(desc),
            HLParserError::IO(e) => HLParserError::IO(e),
        })?;

        Ok(journal)
    }
}

impl<'a> From<nom::error::Error<&'a str>> for HLParserError<&'a str> {
    fn from(err: nom::error::Error<&'a str>) -> Self {
        HLParserError::Parse(err.input, err.code)
    }
}

impl<I, E> FromExternalError<I, E> for HLParserError<I> {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        HLParserError::Parse(input, kind)
    }
}

impl<I> ParseError<I> for HLParserError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        HLParserError::Parse(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::types::AmountSign;

    use super::{Amount, Description, Posting, Status, Transaction};

    #[test]
    fn test_amount_sign_multiplier() {
        assert_eq!(AmountSign::Plus.multiplier(), 1);
        assert_eq!(AmountSign::Minus.multiplier(), -1);
    }

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
                        value: 1,
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

        assert!(transaction.validate::<&str>().is_ok());
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
                        value: 1,
                    }),
                    status: Status::Unmarked,
                },
                Posting {
                    account_name: "income:salary".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: 0,
                    }),
                    status: Status::Unmarked,
                },
            ],
            tags: vec![],
        };

        assert!(transaction.validate::<&str>().is_err());
    }
}
