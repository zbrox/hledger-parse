use winnow::{
    ascii::{line_ending, space0, space1, till_line_ending},
    combinator::{alt, delimited, eof, opt, separated_pair, terminated},
    PResult, Parser,
};

use crate::amount::parsers::{parse_currency_string, parse_money_amount};

use super::types::Commodity;

fn parse_commodity_directive_single_line(input: &mut &str) -> PResult<Commodity> {
    let _ = terminated("commodity", space1).parse_next(input)?;
    alt((
        separated_pair(
            parse_money_amount.with_recognized(),
            space0,
            parse_currency_string,
        )
        .with_recognized()
        .map(|((_, name), full_format)| Commodity {
            name: name.to_string(),
            format: Some(full_format.to_string()),
        }),
        separated_pair(
            parse_currency_string,
            space0,
            opt(parse_money_amount.with_recognized()),
        )
        .with_recognized()
        .map(|((name, format), full_format)| Commodity {
            name: name.to_string(),
            format: format.map(|_| full_format.to_string()),
        }),
    ))
    .parse_next(input)
}

fn parse_commodity_directive_multi_line(input: &mut &str) -> PResult<Commodity> {
    let _ = terminated("commodity", space1).parse_next(input)?;
    let name = terminated(parse_currency_string, line_ending).parse_next(input)?;
    let _ = delimited(space1, "format", space1).parse_next(input)?;
    let full_format = till_line_ending(input)?;

    Ok(Commodity {
        name: name.to_string(),
        format: Some(full_format.to_string()),
    })
}

// TODO: add commodity directive comments
pub fn parse_commodity_directive(input: &mut &str) -> PResult<Commodity> {
    terminated(
        alt((
            parse_commodity_directive_multi_line,
            parse_commodity_directive_single_line,
        )),
        alt((line_ending, eof)),
    )
    .parse_next(input)
}
