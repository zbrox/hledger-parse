use winnow::{
    ascii::{space0, till_line_ending},
    combinator::preceded,
    token::one_of,
    Parser, Result as PResult,
};

pub fn parse_line_comment<'s>(input: &mut &'s str) -> PResult<&'s str> {
    preceded(one_of(['#', ';', '*']), preceded(space0, till_line_ending)).parse_next(input)
}

pub fn parse_transaction_comment<'s>(input: &mut &'s str) -> PResult<&'s str> {
    let _ = space0.parse_next(input)?;
    preceded(';', preceded(space0, till_line_ending)).parse_next(input)
}
