use nom::{
    branch::alt,
    character::complete::{line_ending, space0},
    combinator::{all_consuming, eof, map, value},
    multi::many_till,
    sequence::terminated,
    Finish,
};

use crate::types::{HLParserError, HLParserIResult, Journal, Transaction};

use super::{comments::parse_line_comment, transactions::parse_transaction};

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Ignore,
    Transaction(Transaction),
}

fn parse_comment_value(input: &str) -> HLParserIResult<&str, Value> {
    value(
        Value::Ignore,
        terminated(parse_line_comment, alt((line_ending, eof))),
    )(input)
}

fn parse_empty_line(input: &str) -> HLParserIResult<&str, Value> {
    value(Value::Ignore, terminated(space0, alt((line_ending, eof))))(input)
}

pub fn parse_journal(input: &str) -> Result<Journal, HLParserError<&str>> {
    let (_, (values, _)) = all_consuming(many_till(
        alt((
            map(parse_transaction, Value::Transaction),
            parse_comment_value,
            parse_empty_line,
        )),
        eof,
    ))(input)
    .finish()?;

    Ok(Journal {
        transactions: values
            .into_iter().filter_map(|v| match v {
                Value::Transaction(t) => Some(t),
                _ => None,
            })
            .collect::<Vec<Transaction>>(),
    })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{
        parsers::journal::{parse_comment_value, parse_empty_line, Value},
        types::{Amount, Description, Journal, Posting, Status, Transaction},
    };

    use super::parse_journal;

    #[test]
    fn test_parse_comment_value() {
        assert_eq!(
            parse_comment_value("; A sample journal file\n").unwrap(),
            ("", Value::Ignore)
        );
        assert_eq!(parse_comment_value(";\n").unwrap(), ("", Value::Ignore));
        assert_eq!(parse_comment_value(";").unwrap(), ("", Value::Ignore));
    }

    #[test]
    fn test_parse_empty_line() {
        assert_eq!(parse_empty_line("\n").unwrap(), ("", Value::Ignore));
        assert_eq!(parse_empty_line("   \n").unwrap(), ("", Value::Ignore));
    }

    #[test]
    fn test_parse_journal_simple() {
        // from https://github.com/simonmichael/hledger/blob/e9c19e12ef62d46f57d3cbbd6814dbcf04bbc508/examples/sample.journal
        let input = r#"; A sample journal file.
;
; Sets up this account tree:
; assets
;   bank
;     checking
;     saving
;   cash
; expenses
;   food
;   supplies
; income
;   gifts
;   salary
; liabilities
;   debts

; declare accounts:
; account assets:bank:checking
; account income:salary
; account income:gifts
; account assets:bank:saving
; account assets:cash
; account expenses:food
; account expenses:supplies
; account liabilities:debts

; declare commodities:
; commodity $

2008/01/01 income
    assets:bank:checking  $1
    income:salary

2008/06/01 gift
    assets:bank:checking  $1
    income:gifts

2008/06/02 save
    assets:bank:saving  $1
    assets:bank:checking

2008/06/03 * eat & shop
    expenses:food      $1
    expenses:supplies  $1
    assets:cash

2008/12/31 * pay off
    liabilities:debts  $1
    assets:bank:checking


;final comment
        "#;
        assert_eq!(
            parse_journal(input).unwrap(),
            Journal {
                transactions: vec![
                    Transaction {
                        primary_date: NaiveDate::from_ymd(2008, 1, 1),
                        secondary_date: None,
                        code: None,
                        status: Status::Unmarked,
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
                    },
                    Transaction {
                        primary_date: NaiveDate::from_ymd(2008, 6, 1),
                        secondary_date: None,
                        code: None,
                        status: Status::Unmarked,
                        description: Description {
                            note: Some("gift".into()),
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
                                account_name: "income:gifts".into(),
                                amount: None,
                                status: Status::Unmarked,
                            },
                        ],
                        tags: vec![],
                    },
                    Transaction {
                        primary_date: NaiveDate::from_ymd(2008, 6, 2),
                        secondary_date: None,
                        code: None,
                        status: Status::Unmarked,
                        description: Description {
                            note: Some("save".into()),
                            payee: None,
                        },
                        postings: vec![
                            Posting {
                                account_name: "assets:bank:saving".into(),
                                amount: Some(Amount {
                                    currency: "$".into(),
                                    value: 1,
                                }),
                                status: Status::Unmarked,
                            },
                            Posting {
                                account_name: "assets:bank:checking".into(),
                                amount: None,
                                status: Status::Unmarked,
                            },
                        ],
                        tags: vec![],
                    },
                    Transaction {
                        primary_date: NaiveDate::from_ymd(2008, 6, 3),
                        secondary_date: None,
                        code: None,
                        status: Status::Cleared,
                        description: Description {
                            note: Some("eat & shop".into()),
                            payee: None,
                        },
                        postings: vec![
                            Posting {
                                account_name: "expenses:food".into(),
                                amount: Some(Amount {
                                    currency: "$".into(),
                                    value: 1,
                                }),
                                status: Status::Unmarked,
                            },
                            Posting {
                                account_name: "expenses:supplies".into(),
                                amount: Some(Amount {
                                    currency: "$".into(),
                                    value: 1,
                                }),
                                status: Status::Unmarked,
                            },
                            Posting {
                                account_name: "assets:cash".into(),
                                amount: None,
                                status: Status::Unmarked,
                            },
                        ],
                        tags: vec![],
                    },
                    Transaction {
                        primary_date: NaiveDate::from_ymd(2008, 12, 31),
                        secondary_date: None,
                        code: None,
                        status: Status::Cleared,
                        description: Description {
                            note: Some("pay off".into()),
                            payee: None,
                        },
                        postings: vec![
                            Posting {
                                account_name: "liabilities:debts".into(),
                                amount: Some(Amount {
                                    currency: "$".into(),
                                    value: 1,
                                }),
                                status: Status::Unmarked,
                            },
                            Posting {
                                account_name: "assets:bank:checking".into(),
                                amount: None,
                                status: Status::Unmarked,
                            },
                        ],
                        tags: vec![],
                    },
                ]
            }
        );
    }
}
