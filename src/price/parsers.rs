use winnow::{
    ascii::{line_ending, space1},
    combinator::{alt, eof, terminated},
    PResult, Parser,
};

use crate::{
    amount::parsers::{parse_amount, parse_currency_string},
    date::parsers::parse_date,
};

use super::types::Price;

pub fn parse_price(input: &mut &str) -> PResult<Price> {
    let _ = terminated("P", space1).parse_next(input)?;
    let (date, _) = terminated(parse_date, space1).parse_next(input)?;
    let commodity = terminated(parse_currency_string, space1).parse_next(input)?;
    let amount = parse_amount(input)?;
    let _ = alt((line_ending, eof)).parse_next(input)?;

    Ok(Price {
        commodity: commodity.into(),
        date,
        amount,
    })
}
