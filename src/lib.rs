mod account;
mod amount;
mod code;
mod comment;
mod commodity;
mod date;
mod description;
mod journal;
mod posting;
mod price;
mod status;
mod tag;
mod transaction;
mod utils;

pub use account::types::Account;
pub use amount::types::Amount;
pub use commodity::types::Commodity;
pub use description::types::Description;
pub use journal::types::Journal;
pub use posting::types::Posting;
pub use price::types::Price;
pub use status::types::Status;
pub use tag::types::Tag;
use thiserror::Error;
pub use transaction::types::Transaction;

pub use journal::parsers::parse_journal;

use journal::types::Value;

#[derive(Error, Debug)]
pub enum HLParserError {
    #[error("IO error: {0}")]
    IO(String),
    #[error("Parse error: {0}")]
    Parse(String), 
    #[error("Validation error: {0}")]
    Validation(ValidationError),
    #[error("Included journal error: {0}")]
    IncludePath(String),
    #[error("Extract error: {0:?}")]
    Extract(Value),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid date components: {}-{}-{}", .0.unwrap_or(0), .1, .2)]
    InvalidDateComponents(Option<i32>, u32, u32),
    #[error("Transaction {0} postings' sum does not equal 0")]
    NonZeroSumTransactionPostings(Transaction),
    #[error("Transaction {0} cannot have more than 1 posting with missing amounts")]
    TransactionWithMissingAmountPostings(Transaction),
    #[error("These accounts are not defined:\n{}", .0.join("\n"))]
    UndefinedAccounts(Vec<String>),
}
