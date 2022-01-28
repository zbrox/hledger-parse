use nom::{
    branch::alt, bytes::complete::take_till, character::complete::char,
    character::complete::space0, sequence::preceded, IResult,
};

use super::utils::is_char_newline;

pub fn parse_comment(input: &str) -> IResult<&str, &str> {
    preceded(
        alt((char('#'), char(';'), char('*'))),
        preceded(space0, take_till(is_char_newline)),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parsers::comments::parse_comment;

    #[test]
    fn test_parse_comment() {
        assert_eq!(parse_comment("; comment\n"), Ok(("\n", "comment")));
        assert_eq!(parse_comment("; comment"), Ok(("", "comment")));
        assert_eq!(parse_comment("* comment\n"), Ok(("\n", "comment")));
        assert_eq!(parse_comment("* comment"), Ok(("", "comment")));
        assert_eq!(parse_comment("# comment\n"), Ok(("\n", "comment")));
        assert_eq!(parse_comment("# comment"), Ok(("", "comment")));
    }
}