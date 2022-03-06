use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{space0, space1},
    combinator::{opt, peek, success, verify},
    sequence::{delimited, pair, preceded, terminated},
};

use crate::types::{Amount, Posting, HLParserIResult};

use super::{amount::parse_amount, status::parse_status};

fn parse_posting_with_amount(input: &str) -> HLParserIResult<&str, (&str, Option<Amount>)> {
    pair(
        verify(
            terminated(take_until("  "), peek(preceded(space1, parse_amount))),
            |s: &str| !s.contains('\n'),
        ),
        opt(parse_amount),
    )(input)
}

fn parse_posting_without_amount(input: &str) -> HLParserIResult<&str, (&str, Option<Amount>)> {
    pair(is_not("\n"), success(None))(input)
}

pub fn parse_posting(input: &str) -> HLParserIResult<&str, Posting> {
    let (tail, (status, (account_name, amount))) = pair(
        delimited(space1, parse_status, space0),
        alt((parse_posting_with_amount, parse_posting_without_amount)),
    )(input)?;

    Ok((
        tail,
        Posting {
            status,
            account_name: account_name.trim().into(),
            amount,
        },
    ))
}

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;

    use crate::{
        parsers::postings::parse_posting,
        types::{Amount, Posting, HLParserError},
    };

    #[test]
    fn test_parse_simple_posting() {
        assert_eq!(
            parse_posting(" assets:cash  $100").unwrap(),
            (
                "",
                Posting {
                    status: crate::types::Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: 100
                    })
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
                    status: crate::types::Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: None
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
                    status: crate::types::Status::Pending,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: 100
                    })
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
                    status: crate::types::Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: None
                }
            )
        )
    }

    #[test]
    fn test_parse_posting_no_starting_space() {
        assert_eq!(
            parse_posting("assets:cash").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                    "assets:cash",
                    ErrorKind::Space
            )).to_string()
        )
    }
}
