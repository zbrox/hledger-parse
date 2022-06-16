use std::fmt::Display;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    description::types::Description, journal::types::Value, posting::types::Posting,
    status::types::Status, tag::types::Tag, HLParserError,
};

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

impl TryInto<Transaction> for Value {
    type Error = HLParserError;

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
            .flat_map(|p| match &p.total_price {
                Some(v) => Some(v.clone()),
                None => p.amount.clone(),
            })
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
