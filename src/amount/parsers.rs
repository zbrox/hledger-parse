use std::str::FromStr;

use rust_decimal::Decimal;
use winnow::{
    ascii::{dec_int, space0},
    combinator::{alt, opt, terminated},
    error::ContextError,
    stream::AsChar,
    token::take_till,
    PResult, Parser,
};

use crate::{
    utils::{in_quotes, is_char_minus},
    // HLParserError, PResult,
};

use super::types::{Amount, AmountSign};

// TODO: no scientific notation parsing
pub fn parse_money_amount(input: &mut &str) -> PResult<Decimal> {
    let (num, _, scale) = (
        dec_int::<_, i64, _>.recognize(),
        opt(alt(('.', ','))),
        opt(dec_int::<_, i64, _>.recognize()),
    )
        .parse_next(input)?;
    let value = format!("{}.{}", num, scale.unwrap_or("0"));
    let value = Decimal::from_str(&value)
        .map_err(|_e| winnow::error::ErrMode::Backtrack(ContextError::new()))?; // TODO: errors

    Ok(value)
}

fn parse_sign(input: &mut &str) -> PResult<Option<AmountSign>> {
    let char = opt(alt(('-', '+'))).parse_next(input)?;
    let sign = match char {
        Some('-') => Some(AmountSign::Minus),
        Some('+') => Some(AmountSign::Plus),
        _ => None,
    };
    Ok(sign)
}

pub fn parse_currency_string<'s>(input: &mut &'s str) -> PResult<&'s str> {
    alt((
        in_quotes,
        take_till(0.., |c: char| {
            c.is_dec_digit() || is_char_minus(c) || c.is_space() || c.is_newline()
        }),
    ))
    .parse_next(input)
}

fn parse_amount_prefix_currency(input: &mut &str) -> PResult<Amount> {
    let sign = terminated(parse_sign, space0).parse_next(input)?;
    let currency = terminated(parse_currency_string, space0).parse_next(input)?;
    let sign = match sign {
        Some(s) => Some(s),
        None => terminated(parse_sign, space0).parse_next(input)?,
    };

    let mut value = parse_money_amount(input)?;
    if let Some(AmountSign::Minus) = sign {
        value.set_sign_negative(true);
    }

    Ok(Amount {
        currency: currency.trim().into(),
        value,
    })
}

fn parse_amount_suffix_currency(input: &mut &str) -> PResult<Amount> {
    let sign = terminated(parse_sign, space0).parse_next(input)?;
    let mut value = terminated(parse_money_amount, space0).parse_next(input)?;
    if let Some(AmountSign::Minus) = sign {
        value.set_sign_negative(true);
    }

    let currency = parse_currency_string(input)?;

    Ok(Amount {
        currency: currency.trim().into(),
        value,
    })
}

pub fn parse_amount(input: &mut &str) -> PResult<Amount> {
    alt((
        parse_amount_suffix_currency, // this needs to go first
        parse_amount_prefix_currency,
    ))
    .parse_next(input)
}
