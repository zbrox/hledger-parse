use std::fmt::Display;

use nom::{branch::alt, character::complete::char, combinator::opt, sequence::terminated};

use crate::HLParserIResult;

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

pub fn parse_status(input: &str) -> HLParserIResult<&str, Status> {
    let (tail, status) = opt(terminated(alt((char('!'), char('*'))), char(' ')))(input)?;

    let status = match status {
        Some('!') => Status::Pending,
        Some('*') => Status::Cleared,
        _ => Status::Unmarked,
    };

    Ok((tail, status))
}

#[cfg(test)]
mod tests {
    use crate::parsers::status::{parse_status, Status};

    #[test]
    fn test_status_cleared() {
        assert_eq!(parse_status("* ").unwrap(), ("", Status::Cleared));
    }

    #[test]
    fn test_status_pending() {
        assert_eq!(parse_status("! ").unwrap(), ("", Status::Pending));
    }

    #[test]
    fn test_status_unmarked() {
        assert_eq!(parse_status("").unwrap(), ("", Status::Unmarked));
        assert_eq!(parse_status(" ").unwrap(), (" ", Status::Unmarked));
    }
}
