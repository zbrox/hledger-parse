use nom::{
    branch::alt,
    character::complete::char,
    character::complete::{line_ending, not_line_ending, space0},
    combinator::{opt, rest},
    multi::many0,
    sequence::{preceded, terminated},
    IResult,
};

pub fn parse_line_comment(input: &str) -> IResult<&str, &str> {
    preceded(
        alt((char('#'), char(';'), char('*'))),
        preceded(space0, not_line_ending),
    )(input)
}

pub fn parse_line_comments(input: &str) -> IResult<&str, Vec<&str>> {
    many0(terminated(parse_line_comment, opt(line_ending)))(input)
}

pub fn parse_transaction_comment(input: &str) -> IResult<&str, &str> {
    preceded(char(';'), preceded(space0, rest))(input)
}

#[cfg(test)]
mod tests {
    use crate::parsers::comments::{parse_line_comment, parse_transaction_comment};

    use super::parse_line_comments;

    #[test]
    fn test_parse_line_comment() {
        assert_eq!(parse_line_comment("; comment"), Ok(("", "comment")));
        assert_eq!(parse_line_comment("* comment"), Ok(("", "comment")));
        assert_eq!(parse_line_comment("# comment"), Ok(("", "comment")));
    }

    #[test]
    fn test_parse_transaction_comment() {
        assert_eq!(parse_transaction_comment("; comment"), Ok(("", "comment")));
        assert_eq!(
            parse_transaction_comment("# comment"),
            Err(nom::Err::Error(nom::error::Error {
                input: "# comment",
                code: nom::error::ErrorKind::Char
            }))
        );
    }

    #[test]
    fn test_parse_multiple_line_comments() {
        let input = r#"; comment1
;comment2
;
;  comment3"#;
        assert_eq!(
            parse_line_comments(input),
            Ok(("", vec!["comment1", "comment2", "", "comment3"]))
        );
    }
}
