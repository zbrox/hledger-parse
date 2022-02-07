use std::cmp::PartialEq;

use chrono::NaiveDate;

#[derive(PartialEq, Debug)]
pub enum Status {
    Unmarked,
    Pending,
    Cleared,
}

#[derive(PartialEq, Debug)]
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

#[derive(PartialEq, Debug)]
pub struct Amount {
    pub currency: String,
    pub value: i32,
}

#[derive(PartialEq, Debug)]
pub struct Posting {
    pub status: Status,
    pub account_name: String,
    pub amount: Option<Amount>,
}

#[derive(PartialEq, Debug)]
pub struct Tag {
    pub name: String,
    pub value: Option<String>,
}

#[derive(PartialEq, Debug)]
pub struct Transaction {
    pub primary_date: NaiveDate,
    pub secondary_date: Option<NaiveDate>,
    pub status: Status,
    pub code: Option<String>,
    pub description: Description,
    pub postings: Vec<Posting>,
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