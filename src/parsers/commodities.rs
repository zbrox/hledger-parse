use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{consumed, eof, map, opt},
    sequence::{delimited, separated_pair, terminated},
};

use crate::{HLParserError, HLParserIResult};

use super::{
    amounts::{parse_currency_string, parse_money_amount},
    journal::Value,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Commodity {
    pub name: String,
    pub format: Option<String>, // TODO: temp before I decide how to store the format properly
}

impl TryInto<Commodity> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Commodity, Self::Error> {
        if let Value::Commodity(c) = self {
            Ok(c)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

fn parse_commodity_directive_single_line(input: &str) -> HLParserIResult<&str, Commodity> {
    let (tail, _) = terminated(tag("commodity"), space1)(input)?;
    alt((
        map(
            consumed(separated_pair(
                consumed(parse_money_amount),
                space0,
                parse_currency_string,
            )),
            |(full_format, (_, name))| Commodity {
                name: name.to_string(),
                format: Some(full_format.to_string()),
            },
        ),
        map(
            consumed(separated_pair(
                parse_currency_string,
                space0,
                opt(consumed(parse_money_amount)),
            )),
            |(full_format, (name, format))| Commodity {
                name: name.to_string(),
                format: format.map(|_| full_format.to_string()),
            },
        ),
    ))(tail)
}

fn parse_commodity_directive_multi_line(input: &str) -> HLParserIResult<&str, Commodity> {
    let (tail, _) = terminated(tag("commodity"), space1)(input)?;
    let (tail, name) = terminated(parse_currency_string, line_ending)(tail)?;
    let (tail, _) = delimited(space1, tag("format"), space1)(tail)?;
    let (tail, full_format) = not_line_ending(tail)?;

    Ok((
        tail,
        Commodity {
            name: name.to_string(),
            format: Some(full_format.to_string()),
        },
    ))
}

// TODO: add commodity directive comments
pub fn parse_commodity_directive(input: &str) -> HLParserIResult<&str, Commodity> {
    terminated(
        alt((
            parse_commodity_directive_multi_line,
            parse_commodity_directive_single_line,
        )),
        alt((line_ending, eof)),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parsers::commodities::{parse_commodity_directive, Commodity};

    #[test]
    fn test_parse_commodity_directive_single_line_name_prefix() {
        assert_eq!(
            parse_commodity_directive("commodity $1000.00").unwrap(),
            (
                "",
                Commodity {
                    name: "$".to_string(),
                    format: Some("$1000.00".to_string()),
                }
            )
        );
        assert_eq!(
            parse_commodity_directive("commodity $ 1000.00").unwrap(),
            (
                "",
                Commodity {
                    name: "$".to_string(),
                    format: Some("$ 1000.00".to_string()),
                }
            )
        );
    }

    #[test]
    fn test_parse_commodity_directive_single_line_name_suffix() {
        assert_eq!(
            parse_commodity_directive("commodity 1000.00USD").unwrap(),
            (
                "",
                Commodity {
                    name: "USD".to_string(),
                    format: Some("1000.00USD".to_string()),
                }
            )
        );
        assert_eq!(
            parse_commodity_directive("commodity 1000.00 USD").unwrap(),
            (
                "",
                Commodity {
                    name: "USD".to_string(),
                    format: Some("1000.00 USD".to_string()),
                }
            )
        )
    }

    #[test]
    fn test_parse_commodity_directive_multi_line() {
        assert_eq!(
            parse_commodity_directive(
                r#"commodity USD
            format 1000.00USD
"#
            )
            .unwrap(),
            (
                "",
                Commodity {
                    name: "USD".to_string(),
                    format: Some("1000.00USD".to_string()),
                }
            )
        );
    }

    #[test]
    fn test_parse_commodity_name_only() {
        assert_eq!(
            parse_commodity_directive("commodity INR").unwrap(),
            (
                "",
                Commodity {
                    name: "INR".to_string(),
                    format: None
                }
            )
        )
    }
}
