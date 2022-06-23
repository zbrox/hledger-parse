use crate::{amount::types::Amount, status::types::Status};

#[derive(PartialEq, Debug, Clone)]
pub struct Posting {
    pub status: Status,
    pub account_name: String,
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}

impl Display for Posting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.amount.as_ref(), self.unit_price.as_ref(), self.total_price.as_ref()) {
            (None, None, None) => write!(f, "  {} {}", self.status, self.account_name),
            (Some(amount), None, None) => write!(f, "  {} {}  {}", self.status, self.account_name, amount),
            (Some(amount), Some(unit_price), None) => write!(f, "  {} {}  {} @ {}", self.status, self.account_name, amount, unit_price),
            (Some(amount), None, Some(total_price)) => write!(f, "  {} {}  {} @@ {}", self.status, self.account_name, amount, total_price),
            _ => unreachable!()
        }
    }
}

#[derive(Clone)]
pub struct PostingComplexAmount {
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}
