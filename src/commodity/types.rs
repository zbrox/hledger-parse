use crate::{journal::types::Value, HLParserError};

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
