use std::{path::PathBuf, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{all_consuming, eof, map, map_res, value},
    multi::many_till,
    sequence::{preceded, terminated, tuple},
    Finish,
};

use crate::{HLParserError, HLParserIResult};

use super::{
    accounts::{parse_account_directive, Account},
    comments::parse_line_comment,
    commodities::{parse_commodity_directive, Commodity},
    prices::{parse_price, Price},
    transactions::{parse_transaction, Transaction},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Ignore,
    Transaction(Transaction),
    Included(Vec<Value>),
    Price(Price),
    Account(Account),
    Commodity(Commodity),
}

#[derive(PartialEq, Debug)]
pub struct Journal {
    pub transactions: Vec<Transaction>,
    pub accounts: Vec<String>,
    pub prices: Vec<Price>,
    pub commodities: Vec<Commodity>,
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
            HLParserError::Extract(v) => HLParserError::Extract(v),
        })?;

        Ok(journal)
    }
}

fn parse_include_statement(input: &str) -> HLParserIResult<&str, PathBuf> {
    let (rest, path) =
        preceded(tuple((tag("include"), space1)), alt((not_line_ending, eof)))(input)?;
    let path = PathBuf::from_str(path)
        .map_err(|_| nom::Err::Error(HLParserError::IncludePath(path.to_owned())))?;
    Ok((rest, path))
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

pub fn parse_journal_contents(
    input: &str,
    base_path: PathBuf,
) -> HLParserIResult<&str, Vec<Value>> {
    all_consuming(map(
        many_till(
            alt((
                map(parse_transaction, Value::Transaction),
                parse_comment_value,
                parse_empty_line,
                map(parse_price, Value::Price),
                map(parse_account_directive, |v| Value::Account(v.into())),
                map(parse_commodity_directive, Value::Commodity),
                map_res::<_, _, _, _, nom::Err<HLParserError>, _, _>(
                    parse_include_statement,
                    |v| {
                        let path: PathBuf = base_path.clone().join(v);
                        if !path.exists() {
                            return Err(nom::Err::Error(HLParserError::IncludePath(
                                path.to_str().unwrap_or("").to_owned(),
                            )));
                        }
                        let included_journal_contents = std::fs::read_to_string(path.clone())
                            .map_err(|e| nom::Err::Error(HLParserError::IO(e)))?;
                        let (_, values) = parse_journal_contents(
                            &included_journal_contents,
                            path.parent()
                                .map(|v| v.to_owned())
                                .expect("Cannot get parent directory"),
                        )?;
                        Ok(Value::Included(values))
                    },
                ),
            )),
            eof,
        ),
        |(v, _)| v,
    ))(input)
}

fn flatten_values(values: Vec<Value>) -> Vec<Value> {
    values
        .into_iter()
        .flat_map(|v| match v {
            Value::Included(contents) => flatten_values(contents),
            _ => vec![v],
        })
        .collect()
}

pub fn parse_journal(input: &str, base_path: Option<PathBuf>) -> Result<Journal, HLParserError> {
    let (_, values) =
        parse_journal_contents(input, base_path.unwrap_or(std::env::current_dir()?)).finish()?;
    let values = flatten_values(values);

    Ok(Journal {
        transactions: values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Transaction>>(),
        accounts: values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Account>>(),
        prices: values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Price>>(),
        commodities: values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Commodity>>(),
    })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use crate::parsers::{
        amounts::Amount,
        descriptions::Description,
        journal::{parse_comment_value, parse_empty_line, Journal, Value},
        postings::Posting,
        status::Status,
        transactions::Transaction,
    };

    use super::{flatten_values, parse_journal};

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
    fn test_flatten_values() {
        let values = vec![
            Value::Transaction(Transaction {
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
            }),
            Value::Included(vec![
                Value::Transaction(Transaction {
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
                }),
                Value::Included(vec![Value::Transaction(Transaction {
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
                })]),
            ]),
        ];
        assert_eq!(
            flatten_values(values),
            [
                Value::Transaction(Transaction {
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
                }),
                Value::Transaction(Transaction {
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
                }),
                Value::Transaction(Transaction {
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
                }),
            ]
        )
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
            parse_journal(input, None).unwrap(),
            Journal {
                accounts: vec![],
                prices: vec![],
                commodities: vec![],
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
                                    value: dec!(1),
                                }),
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
                            },
                            Posting {
                                account_name: "income:gifts".into(),
                                amount: None,
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
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
                                    value: dec!(1),
                                }),
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
                            },
                            Posting {
                                account_name: "assets:bank:checking".into(),
                                amount: None,
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
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
                                    value: dec!(1),
                                }),
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
                            },
                            Posting {
                                account_name: "expenses:supplies".into(),
                                amount: Some(Amount {
                                    currency: "$".into(),
                                    value: dec!(1),
                                }),
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
                            },
                            Posting {
                                account_name: "assets:cash".into(),
                                amount: None,
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
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
                                    value: dec!(1),
                                }),
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
                            },
                            Posting {
                                account_name: "assets:bank:checking".into(),
                                amount: None,
                                status: Status::Unmarked,
                                unit_price: None,
                                total_price: None,
                            },
                        ],
                        tags: vec![],
                    },
                ]
            }
        );
    }
}
