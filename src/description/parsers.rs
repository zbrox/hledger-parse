use winnow::{
    ascii::{line_ending, space0},
    combinator::{alt, delimited, eof, opt, peek, repeat_till, terminated},
    token::{any, take_until},
    PResult, Parser,
};

use super::types::Description;

fn description_end<'s>(input: &mut &'s str) -> PResult<&'s str> {
    alt((line_ending, eof, delimited(space0, ";", space0))).parse_next(input)
}

fn parse_only_note<'s>(input: &mut &'s str) -> PResult<&'s str> {
    repeat_till::<_, char, Vec<char>, _, _, _, _>(0.., any, peek(description_end))
        .take()
        .parse_next(input)
}

pub fn parse_description(input: &mut &str) -> PResult<Description> {
    let payee = opt(terminated(take_until(0.., "|"), "|")).parse_next(input)?;
    let note = opt(parse_only_note).parse_next(input)?;

    Ok(Description {
        payee: payee
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        note: note
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
    })
}
