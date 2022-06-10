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

use crate::{HLParserError, HLParserIResult};

use super::utils::{in_quotes, is_char_digit, is_char_minus, is_char_newline, is_char_space};

#[derive(PartialEq)]
pub enum AmountSign {
    Plus,
    Minus,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Amount {
    pub currency: String,
    pub value: Decimal,
}

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

fn parse_sign(input: &str) -> HLParserIResult<&str, AmountSign> {
    let (tail, char) = opt(char('-'))(input)?;
    let sign = match char {
        Some(_) => AmountSign::Minus,
        None => AmountSign::Plus,
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
    let (tail, currency) = parse_currency_string(tail)?;
    let (tail, sign) = match sign {
        AmountSign::Minus => (tail, sign),
        AmountSign::Plus => terminated(parse_sign, space0)(tail)?,
    };

    let (tail, mut value) = parse_money_amount(tail)?;
    if sign == AmountSign::Minus {
        value.set_sign_negative(true);
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
    if sign == AmountSign::Minus {
        value.set_sign_negative(true);
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

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::parsers::amounts::{parse_amount, Amount};

    use super::{parse_currency_string, parse_money_amount};

    #[test]
    fn test_parse_amount_currency_prefixed() {
        assert_eq!(
            parse_amount("$100").unwrap(),
            (
                "",
                Amount {
                    currency: "$".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("$ 100").unwrap(),
            (
                "",
                Amount {
                    currency: "$".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("$  100").unwrap(),
            (
                "",
                Amount {
                    currency: "$".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("\"silver coins\" 100").unwrap(),
            (
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("\"silver coins\"  100").unwrap(),
            (
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: dec!(100)
                }
            )
        );
    }

    #[test]
    fn test_parse_amount_currency_suffixed() {
        assert_eq!(
            parse_amount("100EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("100 EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("100  EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("100 \"silver coins\"").unwrap(),
            (
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: dec!(100)
                }
            )
        );
        assert_eq!(
            parse_amount("100  \"silver coins\"").unwrap(),
            (
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: dec!(100)
                }
            )
        );
    }

    #[test]
    fn test_parse_amount_negative_currency_suffixed() {
        assert_eq!(
            parse_amount("-100EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("- 100EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("-  100EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("-100 EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("-100  EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("- 100 EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("-  100 EUR").unwrap(),
            (
                "",
                Amount {
                    currency: "EUR".into(),
                    value: dec!(-100)
                }
            )
        );
    }

    #[test]
    fn test_parse_amount_negative_currency_prefixed() {
        assert_eq!(
            parse_amount("-$100").unwrap(),
            (
                "",
                Amount {
                    currency: "$".into(),
                    value: dec!(-100)
                }
            )
        );
        assert_eq!(
            parse_amount("$-100").unwrap(),
            (
                "",
                Amount {
                    currency: "$".into(),
                    value: dec!(-100)
                }
            )
        );
    }

    #[test]
    fn test_parse_money_amount_int() {
        assert_eq!(parse_money_amount("100EUR").unwrap(), ("EUR", dec!(100)))
    }

    #[test]
    fn test_parse_money_amount_double() {
        assert_eq!(parse_money_amount("100.00EUR").unwrap(), ("EUR", dec!(100)));
        assert_eq!(parse_money_amount("100,00EUR").unwrap(), ("EUR", dec!(100)));
    }

    #[test]
    fn test_parse_money_amount_small() {
        assert_eq!(
            parse_money_amount("0.007EUR").unwrap(),
            ("EUR", dec!(0.007))
        );
    }

    #[test]
    fn test_parse_fractional_amount() {
        assert_eq!(
            parse_amount("$100.95").unwrap(),
            (
                "",
                Amount {
                    currency: "$".into(),
                    value: dec!(100.95),
                }
            )
        )
    }

    #[test]
    fn test_parse_currency_string_symbol() {
        assert_eq!(parse_currency_string("$").unwrap(), ("", "$"));
        assert_eq!(parse_currency_string("$ ").unwrap(), (" ", "$"));
    }

    #[test]
    fn test_parse_currency_string_quotes() {
        assert_eq!(
            parse_currency_string("\"Imaginary money\"").unwrap(),
            ("", "Imaginary money")
        );
        assert_eq!(
            parse_currency_string("\"Imaginary money\" ").unwrap(),
            (" ", "Imaginary money")
        );
    }

    #[test]
    fn test_parse_currency_string_iso() {
        assert_eq!(parse_currency_string("USD").unwrap(), ("", "USD"));
        assert_eq!(parse_currency_string("USD ").unwrap(), (" ", "USD"));
    }
}
