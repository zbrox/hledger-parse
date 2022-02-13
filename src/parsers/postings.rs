use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{not_line_ending, space0, space1},
    combinator::{opt, peek},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use crate::types::Posting;

use super::{amount::parse_amount, status::parse_status};

pub fn parse_posting(input: &str) -> IResult<&str, Posting> {
    let (tail, (status, account_name, amount)) = tuple((
        delimited(space1, parse_status, space0),
        alt((
            terminated(take_until("  "), peek(preceded(space1, parse_amount))),
            not_line_ending,
        )),
        opt(parse_amount),
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
            parse_posting(" assets:cash  $100"),
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
            parse_posting(" ! assets:cash  $100"),
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
            parse_posting(" assets:cash"),
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
            parse_posting("assets:cash"),
            Err(nom::Err::Error(nom::error::Error {
                input: "assets:cash",
                code: ErrorKind::Space
            }))
        )
    }
}
