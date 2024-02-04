use chrono::NaiveDate;
use rust_decimal_macros::dec;

use crate::{
    amount::types::Amount, description::types::Description, posting::types::Posting,
    status::types::Status, tag::types::Tag,
};

use super::{
    parsers::{parse_comments_tags, parse_transaction},
    types::Transaction,
};

#[test]
fn parse_comment_with_tags() {
    let mut input = " a comment containing tag1:, tag2: some value";
    assert_eq!(
        parse_comments_tags(&mut input).unwrap(),
        (
            "a comment containing",
            vec![
                Tag {
                    name: "tag1".into(),
                    value: None,
                },
                Tag {
                    name: "tag2".into(),
                    value: Some("some value".into()),
                },
            ],
        )
    );
}

#[test]
fn test_simple_transaction() {
    let mut input = r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
}

#[test]
fn test_empty_description_cleared_transaction() {
    let mut input = r#"2008/01/01 * |
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
}

#[test]
fn test_empty_description_unmarked_transaction() {
    let mut input = r#"2008/01/01 |
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
}

#[test]
fn test_transaction_ending_after_postings() {
    let mut input = r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary

2008/06/01 gift
    assets:bank:checking  $1
    income:gifts"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
                    status: Status::Unmarked,
                    amount: None,
                    unit_price: None,
                    total_price: None,
                },
            ],
        }
    );
    assert_eq!(
        input,
        "\n2008/06/01 gift\n    assets:bank:checking  $1\n    income:gifts",
    );
}

#[test]
fn test_simple_transaction_with_empty_amount_posting() {
    let mut input = r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
                    status: Status::Unmarked,
                    amount: None,
                    unit_price: None,
                    total_price: None,
                },
            ],
        }
    );
    assert_eq!(input, "");
}

#[test]
fn test_transaction_with_code() {
    let mut input = r#"2008/01/01 (101) income
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1)
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
}

#[test]
fn test_transaction_with_status() {
    let mut input = r#"2008/01/01 * (101) income
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1)
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
}

#[test]
fn test_transaction_no_description() {
    let mut input = r#"2008/01/01
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1)
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
}

#[test]
fn test_transaction_with_tags() {
    let mut input = r#"2008/01/01 ; some comment tag1:value1, tag2:value2, tag3:
    assets:bank:checking   $1
    income:salary         $-1
"#;
    assert_eq!(
        parse_transaction(&mut input).unwrap(),
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
                    account: "assets:bank:checking".into(),
                    status: Status::Unmarked,
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1)
                    }),
                    unit_price: None,
                    total_price: None,
                },
                Posting {
                    account: "income:salary".into(),
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
    );
    assert_eq!(input, "");
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
                account: "assets:bank:checking".into(),
                amount: Some(Amount {
                    currency: "$".into(),
                    value: dec!(1),
                }),
                status: Status::Unmarked,
                unit_price: None,
                total_price: None,
            },
            Posting {
                account: "income:salary".into(),
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
                account: "assets:bank:checking".into(),
                amount: Some(Amount {
                    currency: "$".into(),
                    value: dec!(1),
                }),
                status: Status::Unmarked,
                unit_price: None,
                total_price: None,
            },
            Posting {
                account: "income:salary".into(),
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
