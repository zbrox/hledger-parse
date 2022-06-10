use nom::{
    bytes::complete::take_until1,
    character::complete::{char, space0},
    sequence::{delimited, pair},
};

use crate::HLParserIResult;

pub fn parse_code(input: &str) -> HLParserIResult<&str, &str> {
    delimited(
        pair(char('('), space0),
        take_until1(")"),
        pair(space0, char(')')),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::{parsers::codes::parse_code, HLParserError};

    #[test]
    fn test_parse_code() {
        assert_eq!(
            parse_code("(code123-1!) rest").unwrap(),
            (" rest", "code123-1!")
        )
    }

    #[test]
    fn test_parse_invalid_code() {
        assert_eq!(
            parse_code("()").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                ")".to_owned(),
                nom::error::ErrorKind::TakeUntil
            ))
            .to_string()
        )
    }
}
