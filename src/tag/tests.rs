use rstest::rstest;
use winnow::error::{ContextError, ErrMode};

use super::{parsers::parse_tag, types::Tag};

#[rstest]
#[case("not a tag:")]
fn test_parse_tag_with_space(#[case] input: &str) {
    let mut input = input;
    let err = parse_tag(&mut input).unwrap_err();
    let expected_err = ErrMode::Backtrack(ContextError::new()); // TODO: errors
    assert_eq!(err, expected_err)
}

#[rstest]
#[case::simple("cash:", "cash")]
#[case::underscore("in_review:", "in_review")]
#[case::dash("in-review:", "in-review")]
#[case::in_quotes("\"in review\":", "in review")]
#[case::unicode("кеш:", "кеш")]
#[case::emoji("👍:", "👍")]
fn test_parse_tag_no_value(#[case] input: &str, #[case] expected: &str) {
    let mut input = input;
    assert_eq!(
        parse_tag(&mut input).unwrap(),
        Tag {
            name: expected.into(),
            value: None,
        }
    );
    assert_eq!(input, "");
}

#[rstest]
#[case::simple("cash:atm", "cash", "atm")]
#[case::underscore("in_review:yes", "in_review", "yes")]
#[case::dash("in-review:no", "in-review", "no")]
#[case::in_quotes("\"in review\":yes", "in review", "yes")]
#[case::unicode("кеш:банкомат", "кеш", "банкомат")]
#[case::emoji("👍:👎", "👍", "👎")]
#[case::space_in_value("cash:atm machine", "cash", "atm machine")]
fn test_parse_tag_with_value(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_value: &str,
) {
    let mut input = input;
    assert_eq!(
        parse_tag(&mut input).unwrap(),
        Tag {
            name: expected_name.into(),
            value: Some(expected_value.into()),
        }
    )
}
