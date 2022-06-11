use std::fmt::Display;

use chrono::NaiveDate;
use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0},
    combinator::opt,
    multi::{many0, separated_list0},
    sequence::terminated,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{HLParserError, HLParserIResult};

use super::{
    codes::parse_code,
    comments::parse_transaction_comment,
    dates::parse_date,
    descriptions::{parse_description, Description},
    journal::Value,
    postings::{parse_posting, Posting},
    status::{parse_status, Status},
    tags::{parse_tag, Tag},
    utils::split_on_space_before_char,
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

fn parse_comments_tags(input: &str) -> HLParserIResult<&str, (Option<&str>, Vec<Tag>)> {
    let (comment_input, tags_input) = split_on_space_before_char(input, ':');
    let comment = match comment_input.trim().len() {
        0 => None,
        _ => Some(comment_input.trim()),
    };
    let (tail, tags) = separated_list0(terminated(tag(","), space0), parse_tag)(tags_input)
        .map_err(nom::Err::convert)?;

    Ok((tail, (comment, tags)))
}

pub fn parse_transaction(input: &str) -> HLParserIResult<&str, Transaction> {
    let (tail, (primary_date, secondary_date)) =
        terminated(parse_date, space0)(input).map_err(nom::Err::convert)?;
    let (tail, status) = parse_status(tail).map_err(nom::Err::convert)?;
    let (tail, code) = opt(parse_code)(tail).map_err(nom::Err::convert)?;
    let (tail, rest_of_line) = terminated(opt(not_line_ending), line_ending)(tail)?;

    // hmmmm...
    let rest_of_line = rest_of_line.unwrap_or("");
    let (description_input, comment_and_tags_input) =
        rest_of_line.split_at(rest_of_line.find(';').unwrap_or(rest_of_line.len()));
    let (_, description) = parse_description(description_input).map_err(nom::Err::convert)?;
    let (_, comment_and_tags) =
        opt(parse_transaction_comment)(comment_and_tags_input).map_err(nom::Err::convert)?;

    let (_, (_comment, tags)) =
        parse_comments_tags(comment_and_tags.unwrap_or("")).map_err(nom::Err::convert)?;
    let (tail, postings) =
        many0(terminated(parse_posting, line_ending))(tail).map_err(nom::Err::convert)?;

    let transaction = Transaction {
        primary_date,
        secondary_date,
        code: code.map(str::to_string),
        status,
        description,
        tags,
        postings,
    };

    transaction.validate().map_err(nom::Err::Error)?;

    Ok((tail, transaction))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use crate::parsers::{
        amounts::Amount, descriptions::Description, postings::Posting, status::Status, tags::Tag,
        transactions::parse_transaction,
    };

    use super::Transaction;

    #[test]
    fn test_simple_transaction() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: Some("income".into()),
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_empty_description_cleared_transaction() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 * |
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: None,
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Cleared,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_empty_description_unmarked_transaction() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 |
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: None,
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_transaction_ending_after_postings() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary

2008/06/01 gift
  assets:bank:checking  $1
  income:gifts"#
            )
            .unwrap(),
            (
                "\n2008/06/01 gift\n  assets:bank:checking  $1\n  income:gifts",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: Some("income".into()),
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: None,
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_simple_transaction_with_empty_amount_posting() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: Some("income".into()),
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: None,
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_transaction_with_code() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 (101) income
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: Some("101".into()),
                    description: Description {
                        note: Some("income".into()),
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_transaction_with_status() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 * (101) income
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: Some("101".into()),
                    description: Description {
                        note: Some("income".into()),
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Cleared,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_transaction_no_description() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: None,
                        payee: None,
                    },
                    tags: vec![],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
    }

    #[test]
    fn test_transaction_with_tags() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 ; some comment tag1:value1, tag2:value2, tag3:
    assets:bank:checking   $1
    income:salary         $-1
"#
            )
            .unwrap(),
            (
                "",
                Transaction {
                    primary_date: NaiveDate::from_ymd(2008, 1, 1),
                    secondary_date: None,
                    code: None,
                    description: Description {
                        note: None,
                        payee: None,
                    },
                    tags: vec![
                        Tag {
                            name: "tag1".into(),
                            value: Some("value1".into())
                        },
                        Tag {
                            name: "tag2".into(),
                            value: Some("value2".into())
                        },
                        Tag {
                            name: "tag3".into(),
                            value: None,
                        }
                    ],
                    status: Status::Unmarked,
                    postings: vec![
                        Posting {
                            account_name: "assets:bank:checking".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(-1)
                            }),
                            unit_price: None,
                            total_price: None,
                        },
                    ],
                }
            )
        )
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
                        value: dec!(1),
                    }),
                    status: Status::Unmarked,
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account_name: "income:salary".into(),
                    amount: None,
                    status: Status::Unmarked,
                    unit_price: None,
                    total_price: None,
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
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account_name: "income:salary".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(0),
                    }),
                    status: Status::Unmarked,
                    unit_price: None,
                    total_price: None,
                },
            ],
            tags: vec![],
        };

        assert!(transaction.validate().is_err());
    }
}
