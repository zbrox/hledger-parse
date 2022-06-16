use crate::{journal::types::Value, HLParserError};

pub type Account = String;

impl TryInto<Account> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Account, Self::Error> {
        if let Value::Account(t) = self {
            Ok(t)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}
