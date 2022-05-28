use nom::{
    bytes::complete::{tag, take_while},
    sequence::terminated,
};

use crate::types::{HLParserIResult, Tag};

use super::utils::is_char_alphanumeric;

pub fn parse_tag(input: &str) -> HLParserIResult<&str, Tag> {
    let (tail, name) = terminated(
        take_while(|c| is_char_alphanumeric(c) || c == '-'),
        tag(":"),
    )(input)?;
    let (tail, value) = take_while(|c| c != ',')(tail)?;
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
    use crate::{
        parsers::tags::parse_tag,
        types::{HLParserError, Tag},
    };

    #[test]
    fn test_parse_tag_with_space() {
        let err = parse_tag("not a tag:").unwrap_err().to_string();
        let expected_err = nom::Err::Error(HLParserError::Parse(
            " a tag:".to_string(),
            nom::error::ErrorKind::Tag,
        ))
        .to_string();
        assert_eq!(err, expected_err,)
    }

    #[test]
    fn test_parse_tag_no_value() {
        assert_eq!(
            parse_tag("cash:").unwrap(),
            (
                "",
                Tag {
                    name: "cash".into(),
                    value: None,
                }
            )
        )
    }

    #[test]
    fn test_parse_tag_with_value() {
        assert_eq!(
            parse_tag("cash:atm").unwrap(),
            (
                "",
                Tag {
                    name: "cash".into(),
                    value: Some("atm".into()),
                }
            )
        )
    }

    #[test]
    fn test_parse_tag_unicode_no_value() {
        assert_eq!(
            parse_tag("кеш:").unwrap(),
            (
                "",
                Tag {
                    name: "кеш".into(),
                    value: None,
                }
            )
        )
    }
}
