use winnow::{ascii::space0, combinator::delimited, token::take_until, PResult, Parser};

pub fn parse_code<'s>(input: &mut &'s str) -> PResult<&'s str> {
    delimited(('(', space0), take_until(1.., ")"), (space0, ')')).parse_next(input)
}
