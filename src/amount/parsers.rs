use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::take_till,
    character::complete::{char, i64, space0, u64},
    combinator::{opt, recognize},
    error::ErrorKind,
    sequence::{terminated, tuple},
};
use rust_decimal::Decimal;

use crate::{
    utils::{in_quotes, is_char_digit, is_char_minus, is_char_newline, is_char_space},
    HLParserError, HLParserIResult,
};

use super::types::{Amount, AmountSign};

// TODO: no scientific notation parsing
pub fn parse_money_amount(input: &str) -> HLParserIResult<&str, Decimal> {
    let (tail, (num, _, scale)) = tuple((
        recognize(i64),
        opt(alt((char('.'), char(',')))),
        opt(recognize(u64)),
    ))(input)?;
    let value = format!("{}.{}", num, scale.unwrap_or("0"));
    let value = Decimal::from_str(&value)
        .map_err(|_| nom::Err::Error(HLParserError::Parse(value.to_string(), ErrorKind::Float)))?;

    Ok((tail, value))
}

fn parse_sign(input: &str) -> HLParserIResult<&str, Option<AmountSign>> {
    let (tail, char) = opt(alt((char('-'), char('+'))))(input)?;
    let sign = match char {
        Some('-') => Some(AmountSign::Minus),
        Some('+') => Some(AmountSign::Plus),
        _ => None,
    };
    Ok((tail, sign))
}

pub fn parse_currency_string(input: &str) -> HLParserIResult<&str, &str> {
    alt((
        in_quotes,
        take_till(|c| {
            is_char_digit(c) || is_char_minus(c) || is_char_space(c) || is_char_newline(c)
        }),
    ))(input)
    .map_err(nom::Err::convert)
}

fn parse_amount_prefix_currency(input: &str) -> HLParserIResult<&str, Amount> {
    let (tail, sign) = terminated(parse_sign, space0)(input)?;
    let (tail, currency) = terminated(parse_currency_string, space0)(tail)?;
    let (tail, sign) = match sign {
        Some(s) => (tail, Some(s)),
        None => terminated(parse_sign, space0)(tail)?,
    };

    let (tail, mut value) = parse_money_amount(tail)?;
    match sign {
        Some(AmountSign::Minus) => value.set_sign_negative(true),
        _ => {}
    }

    Ok((
        tail,
        Amount {
            currency: currency.trim().into(),
            value,
        },
    ))
}

fn parse_amount_suffix_currency(input: &str) -> HLParserIResult<&str, Amount> {
    let (tail, sign) = terminated(parse_sign, space0)(input)?;
    let (tail, mut value) = terminated(parse_money_amount, space0)(tail)?;
    match sign {
        Some(AmountSign::Minus) => value.set_sign_negative(true),
        _ => {}
    }

    let (tail, currency) = parse_currency_string(tail)?;

    Ok((
        tail,
        Amount {
            currency: currency.trim().into(),
            value,
        },
    ))
}

pub fn parse_amount(input: &str) -> HLParserIResult<&str, Amount> {
    alt((
        parse_amount_suffix_currency, // this needs to go first
        parse_amount_prefix_currency,
    ))(input)
}
