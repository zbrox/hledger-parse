use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{space0, space1},
    combinator::{map_res, opt, peek, success, verify},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use crate::{HLParserError, HLParserIResult};

use super::{
    amounts::{parse_amount, Amount},
    status::{parse_status, Status},
};

#[derive(PartialEq, Debug, Clone)]
pub struct Posting {
    pub status: Status,
    pub account_name: String,
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}

#[derive(Clone)]
struct PostingComplexAmount {
    amount: Option<Amount>,
    unit_price: Option<Amount>,
    total_price: Option<Amount>,
}

fn parse_posting_with_amount(input: &str) -> HLParserIResult<&str, (&str, PostingComplexAmount)> {
    let (tail, account_name) = verify(
        terminated(take_until("  "), peek(preceded(space1, parse_amount))),
        |s: &str| !s.contains('\n'),
    )(input)?;
    let (tail, complex_amount) = alt((
        // order of parsers is important
        map_res(
            separated_pair(
                parse_amount,
                delimited(space0, tag("@@"), space0),
                parse_amount,
            ),
            |(amount, total_price)| -> Result<PostingComplexAmount, HLParserError> {
                Ok(PostingComplexAmount {
                    amount: Some(amount),
                    unit_price: None,
                    total_price: Some(total_price),
                })
            },
        ),
        map_res(
            separated_pair(
                parse_amount,
                delimited(space0, tag("@"), space0),
                parse_amount,
            ),
            |(amount, unit_price)| -> Result<PostingComplexAmount, HLParserError> {
                Ok(PostingComplexAmount {
                    amount: Some(amount),
                    unit_price: Some(unit_price),
                    total_price: None,
                })
            },
        ),
        map_res(
            opt(parse_amount),
            |amount| -> Result<PostingComplexAmount, HLParserError> {
                Ok(PostingComplexAmount {
                    amount,
                    unit_price: None,
                    total_price: None,
                })
            },
        ),
    ))(tail)?;

    Ok((tail, (account_name, complex_amount)))
}

fn parse_posting_without_amount(
    input: &str,
) -> HLParserIResult<&str, (&str, PostingComplexAmount)> {
    tuple((
        is_not("\n"),
        success(PostingComplexAmount {
            amount: None,
            unit_price: None,
            total_price: None,
        }),
    ))(input)
}

pub fn parse_posting(input: &str) -> HLParserIResult<&str, Posting> {
    let (tail, (status, (account_name, complex_amount))) = pair(
        delimited(space1, parse_status, space0),
        alt((parse_posting_with_amount, parse_posting_without_amount)),
    )(input)?;

    Ok((
        tail,
        Posting {
            status,
            account_name: account_name.trim().into(),
            amount: complex_amount.amount,
            unit_price: complex_amount.unit_price,
            total_price: complex_amount.total_price,
        },
    ))
}

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;
    use rust_decimal_macros::dec;

    use crate::{
        parsers::{amounts::Amount, postings::parse_posting, status::Status},
        HLParserError,
    };

    use super::Posting;

    #[test]
    fn test_parse_simple_posting() {
        assert_eq!(
            parse_posting(" assets:cash  $100").unwrap(),
            (
                "",
                Posting {
                    status: Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(100),
                    }),
                    unit_price: None,
                    total_price: None,
                }
            )
        )
    }

    #[test]
    fn test_correct_termination_parse_posting() {
        assert_eq!(
            parse_posting(" assets:cash\n2008/06/01 gift\n  assets:bank:checking  $1").unwrap(),
            (
                "\n2008/06/01 gift\n  assets:bank:checking  $1",
                Posting {
                    status: Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: None,
                    unit_price: None,
                    total_price: None,
                }
            )
        )
    }

    #[test]
    fn test_parse_posting_with_status() {
        assert_eq!(
            parse_posting(" ! assets:cash  $100").unwrap(),
            (
                "",
                Posting {
                    status: Status::Pending,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(100)
                    }),
                    unit_price: None,
                    total_price: None,
                }
            )
        )
    }

    #[test]
    fn test_parse_posting_without_amount() {
        assert_eq!(
            parse_posting(" assets:cash").unwrap(),
            (
                "",
                Posting {
                    status: Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: None,
                    unit_price: None,
                    total_price: None,
                }
            )
        )
    }

    #[test]
    fn test_parse_posting_no_starting_space() {
        assert_eq!(
            parse_posting("assets:cash").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "assets:cash".to_string(),
                ErrorKind::Space
            ))
            .to_string()
        )
    }

    #[test]
    fn test_parse_posting_with_unit_price() {
        assert_eq!(
            parse_posting(" ! assets:cash  $100 @ EUR0.94").unwrap(),
            (
                "",
                Posting {
                    status: Status::Pending,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(100)
                    }),
                    unit_price: Some(Amount {
                        currency: "EUR".into(),
                        value: dec!(0.94),
                    }),
                    total_price: None,
                }
            )
        )
    }

    #[test]
    fn test_parse_posting_with_total_price() {
        assert_eq!(
            parse_posting(" ! assets:cash  $100 @@ €93,89").unwrap(),
            (
                "",
                Posting {
                    status: Status::Pending,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(100)
                    }),
                    unit_price: None,
                    total_price: Some(Amount {
                        currency: "€".into(),
                        value: dec!(93.89),
                    }),
                }
            )
        )
    }
}
