use winnow::{
    combinator::{alt, terminated},
    token::take_while,
    PResult, Parser,
};

use crate::utils::in_quotes;

use super::types::Tag;

fn parse_tag_name_no_space<'s>(input: &mut &'s str) -> PResult<&'s str> {
    take_while(0.., |c| c != ' ' && c != '\t' && c != ':').parse_next(input)
}

pub fn parse_tag(input: &mut &str) -> PResult<Tag> {
    let name = terminated(alt((in_quotes, parse_tag_name_no_space)), ":").parse_next(input)?;
    let value = take_while(0.., |c| c != ',').parse_next(input)?;
    let value = value.trim();
    let value = match value.len() {
        0 => None,
        _ => Some(value),
    };

    Ok(Tag {
        name: name.into(),
        value: value.map(str::trim).map(str::to_string),
    })
}
