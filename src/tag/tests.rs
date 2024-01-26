use rstest::rstest;

use crate::HLParserError;

use super::{parsers::parse_tag, types::Tag};

#[rstest]
#[case("not a tag:", " a tag:")]
fn test_parse_tag_with_space(#[case] input: &str, #[case] expected: &str) {
    let err = parse_tag(input).unwrap_err().to_string();
    let expected_err = nom::Err::Error(HLParserError::Parse(
        expected.to_string(),
        nom::error::ErrorKind::Tag,
    ))
    .to_string();
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
    assert_eq!(
        parse_tag(input).unwrap(),
        (
            "",
            Tag {
                name: expected.into(),
                value: None,
            }
        )
    )
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
    assert_eq!(
        parse_tag(input).unwrap(),
        (
            "",
            Tag {
                name: expected_name.into(),
                value: Some(expected_value.into()),
            }
        )
    )
}
