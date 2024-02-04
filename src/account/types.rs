use std::fmt::Display;

use crate::{journal::types::Value, HLParserError};

/// A ledger account
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Account(String);

impl Account {
    /// Returns the full path components of an account
    pub fn components(&self) -> Vec<String> {
        self.0.split(':').map(|v| v.to_string()).collect()
    }

    /// Returns true if the account is a child of the given account
    pub fn is_child_of(&self, account: &Account) -> bool {
        self.0.starts_with(&account.to_string())
    }
}

impl TryInto<Account> for Value {
    type Error = HLParserError<&'static str>;

    fn try_into(self) -> Result<Account, Self::Error> {
        if let Value::Account(t) = self {
            Ok(t)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

impl From<String> for Account {
    fn from(value: String) -> Self {
        Account(value)
    }
}

impl From<&str> for Account {
    fn from(value: &str) -> Self {
        Account(value.into())
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
