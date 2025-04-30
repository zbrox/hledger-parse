use winnow::{
    ascii::{space0, till_line_ending},
    combinator::{alt, preceded},
    Parser, Result as PResult,
};

pub fn parse_line_comment<'s>(input: &mut &'s str) -> PResult<&'s str> {
    preceded(alt(('#', ';', '*')), preceded(space0, till_line_ending)).parse_next(input)
}

pub fn parse_transaction_comment<'s>(input: &mut &'s str) -> PResult<&'s str> {
    preceded(';', preceded(space0, till_line_ending)).parse_next(input)
}
