use std::fmt::Display;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    description::types::Description, journal::types::Value, posting::types::Posting,
    status::types::Status, tag::types::Tag, HLParserError, ValidationError,
};

/// Transaction information
///
/// # Example
///
/// ```
/// use rust_decimal_macros::dec;
/// use chrono::NaiveDate;
/// use hledger_parse::{Amount, Description, Posting, Status, Tag, Transaction};
///
/// let transaction = Transaction {
///     primary_date: NaiveDate::from_ymd(2022, 6, 23),
///     secondary_date: None,
///     status: Status::Cleared,
///     code: Some("12345".to_string()),
///     description: Description {
///         payee: Some("Cheers".to_string()),
///         note: None,
///     },
///     postings: vec![
///         Posting {
///             status: Status::Unmarked,
///             account: "assets:cash".into(),
///             amount: Some(Amount {
///                 currency: "EUR".into(),
///                 value: dec!(-5),
///             }),
///             unit_price: None,
///             total_price: None,
///         },
///         Posting {
///             status: Status::Unmarked,
///             account: "expenses:bars".into(),
///             amount: Some(Amount {
///                 currency: "EUR".into(),
///                 value: dec!(5),
///             }),
///             unit_price: None,
///             total_price: None,
///         },
///     ],
///     tags: vec![
///         Tag {
///             name: "tag1".to_string(),
///             value: Some("some value".to_string()),
///         },
///         Tag {
///             name: "tag2".to_string(),
///             value: None,
///         },
///     ]
/// };
/// assert_eq!(r#"2022-06-23 * Cheers | ; tag1:some value, tag2:
///    assets:cash  -5 EUR
///    expenses:bars  5 EUR
/// "#, format!("{}", transaction));
/// ```
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Transaction {
    /// The primary date of the transaction
    pub primary_date: NaiveDate,
    /// The optional secondary date, e.g. the clearance date
    pub secondary_date: Option<NaiveDate>,
    /// The status of the transaction
    pub status: Status,
    /// The optional code of the transaction
    pub code: Option<String>,
    /// The description of the transaction
    pub description: Description,
    /// The postings of the transaction
    pub postings: Vec<Posting>,
    /// The tags of the transaction
    pub tags: Vec<Tag>,
}

impl TryInto<Transaction> for Value {
    type Error = HLParserError<&'static str>;

    fn try_into(self) -> Result<Transaction, Self::Error> {
        if let Value::Transaction(t) = self {
            Ok(t)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.secondary_date {
            Some(sec_date) => write!(f, "{}={}", self.primary_date, sec_date)?,
            None => write!(f, "{} ", self.primary_date)?,
        };
        write!(f, "{}", self.status)?;
        if self.status != Status::Unmarked {
            write!(f, " ")?;
        }
        write!(f, "{}", self.description)?;
        if !self.tags.is_empty() {
            write!(f, " ; ")?;
            write!(
                f,
                "{}",
                self.tags
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
        writeln!(f)?;
        for p in &self.postings {
            writeln!(f, "{}", p)?;
        }
        Ok(())
    }
}

impl Transaction {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_postings()?;
        Ok(())
    }

    fn validate_postings(&self) -> Result<(), ValidationError> {
        let none_amounts = self.postings.iter().filter(|p| p.amount.is_none()).count();

        if none_amounts > 1_usize {
            return Err(ValidationError::TransactionWithMissingAmountPostings(self.clone()));
        }

        if none_amounts == 1_usize {
            return Ok(());
        }

        let postings_sum = self
            .postings
            .iter()
            .flat_map(|p| match &p.total_price {
                Some(v) => Some(v.clone()),
                None => p.amount.clone(),
            })
            .map(|a| a.value) // TODO: different currencies, conversion rates
            .sum::<Decimal>();

        if postings_sum != dec!(0) {
            return Err(ValidationError::NonZeroSumTransactionPostings(self.clone()));
        }

        Ok(())
    }
}
