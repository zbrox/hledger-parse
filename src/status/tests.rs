use crate::status::{parsers::parse_status, types::Status};

#[test]
fn test_status_cleared() {
    let mut input = "* ";
    assert_eq!(parse_status(&mut input).unwrap(), Status::Cleared);
    assert_eq!(input, "");
}

#[test]
fn test_status_pending() {
    let mut input = "! ";
    assert_eq!(parse_status(&mut input).unwrap(), Status::Pending);
    assert_eq!(input, "");
}

#[test]
fn test_status_unmarked() {
    let mut input = "";
    assert_eq!(parse_status(&mut input).unwrap(), Status::Unmarked);
    assert_eq!(input, "");
    let mut input = " ";
    assert_eq!(parse_status(&mut input).unwrap(), Status::Unmarked);
    assert_eq!(input, " ");
}
