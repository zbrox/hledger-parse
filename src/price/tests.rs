use chrono::NaiveDate;
use rust_decimal_macros::dec;

use crate::amount::types::Amount;

use super::{parsers::parse_price, types::Price};

#[test]
fn test_valid_price() {
    let mut input = "P 2017-01-01 EUR SEK 9.552532877";
    assert_eq!(
        parse_price(&mut input).unwrap(),
        Price {
            commodity: "EUR".to_string(),
            date: NaiveDate::from_ymd(2017, 1, 1),
            amount: Amount {
                currency: "SEK".to_string(),
                value: dec!(9.552532877),
            }
        }
    )
}
