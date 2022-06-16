use chrono::NaiveDate;

use crate::{amount::types::Amount, journal::types::Value, HLParserError};

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
