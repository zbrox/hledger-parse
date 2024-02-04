use rstest::rstest;
use winnow::error::{AddContext, ContextError, ErrMode};

use crate::account::parsers::parse_account_directive;

#[rstest]
#[case("account assets:cash", "", "assets:cash")]
#[case("account    assets:cash", "", "assets:cash")]
fn test_parse_account_directive(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_account_name: &str,
) {
    let mut input = input;
    assert_eq!(
        parse_account_directive(&mut input).unwrap(),
        expected_account_name
    );
    assert_eq!(input, expected_remaining);
}

#[test]
fn test_parse_invalid_account_directive() {
    parse_account_directive(&mut "account assets:cash  ").unwrap_err();
    assert_eq!(
        parse_account_directive(&mut "account assets:cash  ").unwrap_err(),
        ErrMode::Backtrack(
            ContextError::new().add_context(&"", winnow::error::StrContext::Label("account name"))
        )
    );
}
