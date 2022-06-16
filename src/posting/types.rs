use crate::{amount::types::Amount, status::types::Status};

#[derive(PartialEq, Debug, Clone)]
pub struct Posting {
    pub status: Status,
    pub account_name: String,
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}

#[derive(Clone)]
pub struct PostingComplexAmount {
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}
