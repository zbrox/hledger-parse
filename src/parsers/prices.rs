use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, space1},
    combinator::eof,
    sequence::terminated,
};

use crate::{HLParserError, HLParserIResult};

use super::{
    amounts::{parse_amount, parse_currency_string, Amount},
    dates::parse_date,
    journal::Value,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Price {
    pub commodity: String,
    pub date: NaiveDate,
    pub amount: Amount,
}

impl TryInto<Price> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Price, Self::Error> {
        if let Value::Price(p) = self {
            Ok(p)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

pub fn parse_price(input: &str) -> HLParserIResult<&str, Price> {
    let (tail, _) = terminated(tag("P"), space1)(input)?;
    let (tail, (date, _)) = terminated(parse_date, space1)(tail)?;
    let (tail, commodity) = terminated(parse_currency_string, space1)(tail)?;
    let (tail, amount) = parse_amount(tail)?;
    let (tail, _) = alt((line_ending, eof))(tail)?;

    Ok((
        tail,
        Price {
            commodity: commodity.into(),
            date,
            amount,
        },
    ))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use crate::parsers::amounts::Amount;

    use super::{parse_price, Price};

    #[test]
    fn test_valid_price() {
        assert_eq!(
            parse_price("P 2017-01-01 EUR SEK 9.552532877").unwrap(),
            (
                "",
                Price {
                    commodity: "EUR".to_string(),
                    date: NaiveDate::from_ymd(2017, 1, 1),
                    amount: Amount {
                        currency: "SEK".to_string(),
                        value: dec!(9.552532877),
                    }
                }
            )
        )
    }
}
