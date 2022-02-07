use nom::{
    bytes::complete::take_until,
    character::{complete::char, complete::space0, is_digit, is_newline, is_space},
    sequence::{delimited, preceded, terminated},
    IResult,
};

pub fn is_char_newline(char: char) -> bool {
    is_newline(char as u8)
}

pub fn is_char_digit(char: char) -> bool {
    is_digit(char as u8)
}

pub fn is_char_minus(char: char) -> bool {
    char == '-'
}

pub fn is_char_space(char: char) -> bool {
    is_space(char as u8)
}

pub fn in_quotes(input: &str) -> IResult<&str, &str> {
    delimited(
        terminated(char('"'), space0),
        take_until("\""),
        preceded(space0, char('"')),
    )(input)
}
