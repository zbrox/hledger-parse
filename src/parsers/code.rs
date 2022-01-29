use nom::{
    bytes::complete::take_until1,
    character::complete::{char, space0},
    sequence::{delimited, pair},
    IResult,
};

fn parse_code(input: &str) -> IResult<&str, &str> {
    delimited(
        pair(char('('), space0),
        take_until1(")"),
        pair(space0, char(')')),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parsers::code::parse_code;

    #[test]
    fn test_parse_code() {
        assert_eq!(parse_code("(code123-1!) rest"), Ok((" rest", "code123-1!")))
    }

    #[test]
    fn test_parse_invalid_code() {
        assert_eq!(parse_code("()"), Err(nom::Err::Error(nom::error::Error::new(")", nom::error::ErrorKind::TakeUntil))))
    }
}
