use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{not_line_ending, space0},
    combinator::opt,
    sequence::{delimited, separated_pair},
    IResult,
};

use crate::types::Description;

fn parse_only_note(input: &str) -> IResult<&str, Option<&str>> {
    opt(alt((not_line_ending, is_not(";"))))(input)
}

fn parse_payee_and_note(input: &str) -> IResult<&str, (Option<&str>, Option<&str>)> {
    separated_pair(
        opt(alt((take_until(" |"), take_until("|")))),
        delimited(space0, tag("|"), space0),
        parse_only_note,
    )(input)
}

pub fn parse_description(input: &str) -> IResult<&str, Description> {
    match parse_payee_and_note(input) {
        Ok((t, (p, n))) => Ok((
            t,
            Description {
                payee: p.map(str::trim).map(str::to_string),
                note: n.map(str::trim).map(str::to_string),
            },
        )),
        Err(_) => match parse_only_note(input) {
            Ok((t, n)) => Ok((
                t,
                Description {
                    payee: None,
                    note: n.map(str::trim).map(str::to_string),
                },
            )),
            Err(e) => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{parsers::descriptions::parse_description, types::Description};

    #[test]
    fn test_parse_description() {
        assert_eq!(
            parse_description("some description"),
            Ok((
                "",
                Description {
                    payee: None,
                    note: Some("some description".into()),
                }
            ))
        )
    }

    #[test]
    fn test_parse_payee_and_note() {
        assert_eq!(
            parse_description("Acme | some description"),
            Ok((
                "",
                Description {
                    payee: Some("Acme".into()),
                    note: Some("some description".into()),
                }
            ))
        )
    }

    #[test]
    fn test_parse_payee_and_note_different_spacing() {
        assert_eq!(
            parse_description("Acme| some description"),
            Ok((
                "",
                Description {
                    payee: Some("Acme".into()),
                    note: Some("some description".into()),
                }
            ))
        );
        assert_eq!(
            parse_description("Acme |some description"),
            Ok((
                "",
                Description {
                    payee: Some("Acme".into()),
                    note: Some("some description".into()),
                }
            ))
        );
        assert_eq!(
            parse_description("Acme|some description"),
            Ok((
                "",
                Description {
                    payee: Some("Acme".into()),
                    note: Some("some description".into()),
                }
            ))
        );
    }
}
