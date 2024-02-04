use winnow::{
    combinator::{alt, opt, terminated},
    PResult, Parser,
};

use super::types::Status;

pub fn parse_status(input: &mut &str) -> PResult<Status> {
    let status = opt(terminated(alt(('!', '*')), ' ')).parse_next(input)?;

    let status = match status {
        Some('!') => Status::Pending,
        Some('*') => Status::Cleared,
        _ => Status::Unmarked,
    };

    Ok(status)
}
