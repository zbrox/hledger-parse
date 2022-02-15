use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0},
    combinator::opt,
    multi::{many0, separated_list0},
    sequence::terminated,
    IResult,
};

use crate::types::{Tag, Transaction};

use super::{
    code::parse_code, comments::parse_transaction_comment, dates::parse_date,
    descriptions::parse_description, postings::parse_posting, status::parse_status,
    tags::parse_tag, utils::split_on_space_before_char,
};

fn parse_comments_tags(input: &str) -> IResult<&str, (Option<&str>, Vec<Tag>)> {
    let (comment_input, tags_input) = split_on_space_before_char(input, ':');
    let comment = match comment_input.trim().len() {
        0 => None,
        _ => Some(comment_input.trim()),
    };
    let (tail, tags) = separated_list0(terminated(tag(","), space0), parse_tag)(tags_input)?;

    Ok((tail, (comment, tags)))
}

pub fn parse_transaction(input: &str) -> IResult<&str, Transaction> {
    let (tail, (primary_date, secondary_date)) = terminated(parse_date, space0)(input)?;
    let (tail, status) = parse_status(tail)?;
    let (tail, code) = opt(parse_code)(tail)?;
    let (tail, rest_of_line) = terminated(opt(not_line_ending), line_ending)(tail)?;

    // hmmmm...
    let rest_of_line = rest_of_line.unwrap_or("");
    let (description_input, comment_and_tags_input) =
        rest_of_line.split_at(rest_of_line.find(';').unwrap_or(rest_of_line.len()));
    let (_, description) = parse_description(description_input)?;
    let (_, comment_and_tags) = opt(parse_transaction_comment)(comment_and_tags_input)?;

    let (_, (_comment, tags)) = parse_comments_tags(comment_and_tags.unwrap_or(""))?;
    let (tail, postings) = many0(terminated(parse_posting, alt((tag("\r\n"), tag("\n")))))(tail)?;

    Ok((
        tail,
        Transaction {
            primary_date,
            secondary_date,
            code: code.map(str::to_string),
            status,
            description,
            tags,
            postings,
        },
    ))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{
        parsers::transactions::parse_transaction,
        types::{Amount, Description, Posting, Status, Tag, Transaction},
    };

    #[test]
    fn test_simple_transaction() {
        assert_eq!(
            parse_transaction(
                r#"2008/01/01 income
    assets:bank:checking   $1
    income:salary         $-1
"#
            ),
            Ok((
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: -1
                            }),
                        },
                    ],
                }
            ))
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
            ),
            Ok((
                "\n2008/01/01 gift\n  assets:bank:checking  $1\n  income:gifts",
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: None,
                        },
                    ],
                }
            ))
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
            ),
            Ok((
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: None,
                        },
                    ],
                }
            ))
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
            ),
            Ok((
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: -1
                            }),
                        },
                    ],
                }
            ))
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
            ),
            Ok((
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: -1
                            }),
                        },
                    ],
                }
            ))
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
            ),
            Ok((
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: -1
                            }),
                        },
                    ],
                }
            ))
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
            ),
            Ok((
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
                                value: 1
                            }),
                        },
                        Posting {
                            account_name: "income:salary".into(),
                            status: Status::Unmarked,
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: -1
                            }),
                        },
                    ],
                }
            ))
        )
    }
}
