use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub enum Status {
    Unmarked,
    Pending,
    Cleared,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Status::Unmarked => write!(f, ""),
            Status::Pending => write!(f, " ! "),
            Status::Cleared => write!(f, " * "),
        }
    }
}
