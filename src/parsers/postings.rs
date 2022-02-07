use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{newline, not_line_ending, space0, space1},
    combinator::opt,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use crate::types::Posting;

use super::{amount::parse_amount, status::parse_status};

pub fn parse_posting(input: &str) -> IResult<&str, Posting> {
    let (tail, (status, account_name, amount)) = tuple((
        delimited(space1, parse_status, space0),
        alt((
            terminated(take_until("  "), terminated(tag("  "), space0)),
            terminated(not_line_ending, newline),
        )),
        opt(terminated(parse_amount, newline)),
    ))(input)?;

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
        types::{Amount, Posting},
    };

    #[test]
    fn test_parse_simple_posting() {
        assert_eq!(
            parse_posting(" assets:cash  $100\n"),
            Ok((
                "",
                Posting {
                    status: crate::types::Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: 100
                    })
                }
            ))
        )
    }

    #[test]
    fn test_parse_posting_with_status() {
        assert_eq!(
            parse_posting(" ! assets:cash  $100\n"),
            Ok((
                "",
                Posting {
                    status: crate::types::Status::Pending,
                    account_name: "assets:cash".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: 100
                    })
                }
            ))
        )
    }

    #[test]
    fn test_parse_posting_without_amount() {
        assert_eq!(
            parse_posting(" assets:cash\n"),
            Ok((
                "",
                Posting {
                    status: crate::types::Status::Unmarked,
                    account_name: "assets:cash".into(),
                    amount: None
                }
            ))
        )
    }

    #[test]
    fn test_parse_posting_no_starting_space() {
        assert_eq!(
            parse_posting("assets:cash\n"),
            Err(nom::Err::Error(nom::error::Error {
                input: "assets:cash\n",
                code: ErrorKind::Space
            }))
        )
    }
}
