use winnow::error::{ContextError, ErrMode};

use super::parsers::parse_code;

#[test]
fn test_parse_code() {
    let mut input = "(code123-1!) rest";
    assert_eq!(parse_code(&mut input).unwrap(), "code123-1!");
    assert_eq!(input, " rest");
}

#[test]
fn test_parse_invalid_code() {
    assert_eq!(
        parse_code(&mut "()").unwrap_err(),
        ErrMode::Backtrack(ContextError::new()) // TODO: errors
    )
}
