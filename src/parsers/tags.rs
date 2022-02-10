use nom::{
    bytes::complete::{tag, take_while},
    combinator::rest,
    sequence::terminated,
    IResult,
};

use crate::types::Tag;

use super::utils::is_char_alphanumeric;

pub fn parse_tag(input: &str) -> IResult<&str, Tag> {
    let (tail, name) = terminated(
        take_while(|c| is_char_alphanumeric(c) || c == '-'),
        tag(":"),
    )(input)?;
    let (tail, value) = rest(tail)?;
    let value = value.trim();
    let value = match value.len() {
        0 => None,
        _ => Some(value),
    };

    Ok((
        tail,
        Tag {
            name: name.into(),
            value: value.map(str::trim).map(str::to_string),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::{parsers::tags::parse_tag, types::Tag};

    #[test]
    fn test_parse_tag_with_space() {
        assert_eq!(
            parse_tag("not a tag:"),
            Err(nom::Err::Error(nom::error::Error {
                input: " a tag:",
                code: nom::error::ErrorKind::Tag
            }))
        )
    }

    #[test]
    fn test_parse_tag_no_value() {
        assert_eq!(
            parse_tag("cash:"),
            Ok((
                "",
                Tag {
                    name: "cash".into(),
                    value: None,
                }
            ))
        )
    }

    #[test]
    fn test_parse_tag_with_value() {
        assert_eq!(
            parse_tag("cash:atm"),
            Ok((
                "",
                Tag {
                    name: "cash".into(),
                    value: Some("atm".into()),
                }
            ))
        )
    }

    #[test]
    fn test_parse_tag_unicode_no_value() {
        assert_eq!(
            parse_tag("кеш:"),
            Ok((
                "",
                Tag {
                    name: "кеш".into(),
                    value: None,
                }
            ))
        )
    }
}
