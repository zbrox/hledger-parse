use nom::{branch::alt, character::complete::char, combinator::opt, sequence::terminated, IResult};

use crate::types::Status;

pub fn parse_status(input: &str) -> IResult<&str, Status> {
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
    use crate::{parsers::status::parse_status, types::Status};

    #[test]
    fn test_status_cleared() {
        assert_eq!(parse_status("* "), Ok(("", Status::Cleared)));
    }

    #[test]
    fn test_status_pending() {
        assert_eq!(parse_status("! "), Ok(("", Status::Pending)));
    }

    #[test]
    fn test_status_unmarked() {
        assert_eq!(parse_status(""), Ok(("", Status::Unmarked)));
        assert_eq!(parse_status(" "), Ok((" ", Status::Unmarked)));
    }
}
