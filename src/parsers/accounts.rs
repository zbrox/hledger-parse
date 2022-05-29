use nom::{
    bytes::complete::tag,
    character::complete::{not_line_ending, space1},
    combinator::verify,
    sequence::{preceded, tuple},
};

use crate::types::HLParserIResult;

pub fn parse_account_directive(input: &str) -> HLParserIResult<&str, &str> {
    verify(
        preceded(tuple((tag("account"), space1)), not_line_ending),
        |account_name: &str| !account_name.contains("  "),
    )(input)
}

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;

    use crate::types::HLParserError;

    use super::parse_account_directive;

    #[test]
    fn test_parse_account_directive() {
        assert_eq!(
            parse_account_directive("account assets:cash").unwrap(),
            ("", "assets:cash")
        );
    }

    #[test]
    fn test_parse_account_directive_invalid_name() {
        assert_eq!(
            parse_account_directive("account assets:cash  ").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "account assets:cash  ".to_string(),
                ErrorKind::Verify
            ))
            .to_string()
        );
    }
}
