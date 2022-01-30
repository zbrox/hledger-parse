use std::cmp::PartialEq;

#[derive(PartialEq, Debug)]
pub enum Status {
    Unmarked,
    Pending,
    Cleared,
}

#[derive(PartialEq, Debug)]
pub struct Description {
    pub payee: Option<String>,
    pub note: Option<String>,
}
