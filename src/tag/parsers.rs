use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    sequence::terminated,
    IResult,
};

use crate::{utils::in_quotes, HLParserIResult};

use super::types::Tag;

fn parse_tag_name_no_space(input: &str) -> IResult<&str, &str> {
    take_while(|c| c != ' ' && c != '\t' && c != ':')(input)
}

pub fn parse_tag(input: &str) -> HLParserIResult<&str, Tag> {
    let (tail, name) = terminated(alt((in_quotes, parse_tag_name_no_space)), tag(":"))(input)
        .map_err(nom::Err::convert)?;
    let (tail, value) = take_while(|c| c != ',')(tail)?;
    let value = value.trim();
    let value = match value.len() {
        0 => None,
        _ => Some(value),
    };

    Ok((
        tail,
        Tag {
            name: name.into(),
            value: value.map(str::trim).map(str::to_string),
        },
    ))
}
