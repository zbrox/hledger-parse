use nom::{
    branch::alt, character::complete::char, character::complete::space0, combinator::rest,
    sequence::preceded, IResult,
};

pub fn parse_line_comment(input: &str) -> IResult<&str, &str> {
    preceded(
        alt((char('#'), char(';'), char('*'))),
        preceded(space0, rest),
    )(input)
}

pub fn parse_transaction_comment(input: &str) -> IResult<&str, &str> {
    preceded(char(';'), preceded(space0, rest))(input)
}

#[cfg(test)]
mod tests {
    use crate::parsers::comments::{parse_line_comment, parse_transaction_comment};

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
}
