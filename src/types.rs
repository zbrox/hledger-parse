use std::cmp::PartialEq;

#[derive(PartialEq, Debug)]
pub enum Status {
    Unmarked,
    Pending,
    Cleared
}
