use nom::{
    branch::alt,
    bytes::complete::take_till,
    character::complete::{char, i32, space0},
    combinator::{opt, rest},
    sequence::terminated,
    IResult,
};

use crate::types::{Amount, AmountSign};

use super::utils::{in_quotes, is_char_digit, is_char_minus};

fn parse_sign(input: &str) -> IResult<&str, AmountSign> {
    let (tail, char) = opt(char('-'))(input)?;
    let sign = match char {
        Some(_) => AmountSign::Minus,
        None => AmountSign::Plus,
    };
    Ok((tail, sign))
}

fn parse_amount_prefix_currency(input: &str) -> IResult<&str, Amount> {
    let (tail, sign) = terminated(parse_sign, space0)(input)?;
    let (tail, currency) = terminated(
        alt((in_quotes, terminated(take_till(|c| is_char_digit(c) || is_char_minus(c)), space0))),
        space0,
    )(tail)?;
    let (tail, sign) = match sign {
        AmountSign::Minus => (tail, sign),
        AmountSign::Plus => terminated(parse_sign, space0)(tail)?,
    };

    let (tail, value) = i32(tail)?;

    Ok((
        tail,
        Amount {
            currency: currency.trim().into(),
            value: value * sign.multiplier() as i32,
        },
    ))
}

fn parse_amount_sufix_currency(input: &str) -> IResult<&str, Amount> {
    let (tail, sign) = terminated(parse_sign, space0)(input)?;
    let (tail, value) = terminated(i32, space0)(tail)?;

    let (tail, currency) = terminated(alt((in_quotes, rest)), space0)(tail)?;

    Ok((
        tail,
        Amount {
            currency: currency.trim().into(),
            value: value * sign.multiplier() as i32,
        },
    ))
}

pub fn parse_amount(input: &str) -> IResult<&str, Amount> {
    alt((
        parse_amount_sufix_currency, // this needs to go first
        parse_amount_prefix_currency,
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::{parsers::amount::parse_amount, types::Amount};

    #[test]
    fn test_parse_amount_currecy_prefixed() {
        assert_eq!(
            parse_amount("$100"),
            Ok((
                "",
                Amount {
                    currency: "$".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("$ 100"),
            Ok((
                "",
                Amount {
                    currency: "$".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("$  100"),
            Ok((
                "",
                Amount {
                    currency: "$".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("\"silver coins\" 100"),
            Ok((
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("\"silver coins\"  100"),
            Ok((
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: 100
                }
            ))
        );
    }

    #[test]
    fn test_parse_amount_currecy_sufixed() {
        assert_eq!(
            parse_amount("100EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("100 EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("100  EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("100 \"silver coins\""),
            Ok((
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: 100
                }
            ))
        );
        assert_eq!(
            parse_amount("100  \"silver coins\""),
            Ok((
                "",
                Amount {
                    currency: "silver coins".into(),
                    value: 100
                }
            ))
        );
    }

    #[test]
    fn test_parse_amount_negative_currency_sufixed() {
        assert_eq!(
            parse_amount("-100EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("- 100EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("-  100EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("-100 EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("-100  EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("- 100 EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("-  100 EUR"),
            Ok((
                "",
                Amount {
                    currency: "EUR".into(),
                    value: -100
                }
            ))
        );
    }

    #[test]
    fn test_parse_amount_negative_currency_prefixed() {
        assert_eq!(
            parse_amount("-$100"),
            Ok((
                "",
                Amount {
                    currency: "$".into(),
                    value: -100
                }
            ))
        );
        assert_eq!(
            parse_amount("$-100"),
            Ok((
                "",
                Amount {
                    currency: "$".into(),
                    value: -100
                }
            ))
        );
    }
}
