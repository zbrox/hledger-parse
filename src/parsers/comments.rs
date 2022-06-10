use nom::{
    branch::alt,
    character::complete::char,
    character::complete::{not_line_ending, space0},
    combinator::{eof, rest},
    sequence::preceded,
};

use crate::HLParserIResult;

pub fn parse_line_comment(input: &str) -> HLParserIResult<&str, &str> {
    preceded(
        alt((char('#'), char(';'), char('*'))),
        preceded(space0, alt((not_line_ending, eof))),
    )(input)
}

pub fn parse_transaction_comment(input: &str) -> HLParserIResult<&str, &str> {
    preceded(char(';'), preceded(space0, rest))(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        parsers::comments::{parse_line_comment, parse_transaction_comment},
        HLParserError,
    };

    #[test]
    fn test_parse_line_comment() {
        assert_eq!(parse_line_comment("; comment").unwrap(), ("", "comment"));
        assert_eq!(parse_line_comment("* comment").unwrap(), ("", "comment"));
        assert_eq!(parse_line_comment("# comment").unwrap(), ("", "comment"));
    }

    #[test]
    fn test_parse_transaction_comment() {
        assert_eq!(
            parse_transaction_comment("; comment").unwrap(),
            ("", "comment")
        );
        assert_eq!(
            parse_transaction_comment("# comment")
                .unwrap_err()
                .to_string(),
            nom::Err::Error(HLParserError::Parse(
                "# comment".to_owned(),
                nom::error::ErrorKind::Char
            ))
            .to_string()
        );
    }

    #[test]
    fn test_parse_line_comment_empty() {
        assert_eq!(parse_line_comment(";").unwrap(), ("", ""));
        assert_eq!(parse_line_comment(";\n").unwrap(), ("\n", ""));
    }
}
