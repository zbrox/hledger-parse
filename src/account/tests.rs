use nom::error::ErrorKind;
use rstest::rstest;

use crate::{account::parsers::parse_account_directive, HLParserError};

#[rstest]
#[case("account assets:cash", "", "assets:cash")]
#[case("account    assets:cash", "", "assets:cash")]
fn test_parse_account_directive(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_account_name: &str,
) {
    assert_eq!(
        parse_account_directive(input).unwrap(),
        (expected_remaining, expected_account_name)
    );
}

#[test]
fn test_parse_account_directive_invalid_name() {
    assert_eq!(
        parse_account_directive("account assets:cash  ")
            .unwrap_err()
            .to_string(),
        nom::Err::Error(HLParserError::Parse(
            "account assets:cash  ".to_string(),
            ErrorKind::Verify
        ))
        .to_string()
    );
}
