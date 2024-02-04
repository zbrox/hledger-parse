use rstest::rstest;
use winnow::error::{ContextError, ErrMode};

use crate::comment::parsers::{parse_line_comment, parse_transaction_comment};

#[rstest]
#[case::line_comment_temporary(";comment", "", "comment")]
#[case::line_comment_temporary_space("; comment", "", "comment")]
#[case::line_comment_permanent("#comment", "", "comment")]
#[case::line_comment_permanent_space("# comment", "", "comment")]
#[case::line_comment_star("*comment", "", "comment")]
#[case::line_comment_star_space("* comment", "", "comment")]
#[case::line_comment_empty(";", "", "")]
#[case::line_comment_empty(";\n", "\n", "")]
fn test_parse_line_comment(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_comment: &str,
) {
    let mut input = input;
    assert_eq!(parse_line_comment(&mut input).unwrap(), expected_comment);
    assert_eq!(input, expected_remaining);
}

#[rstest]
#[case::transaction_comment(";comment", "", "comment")]
#[case::transaction_comment_space("; comment", "", "comment")]
#[case::transaction_comment_multiword(";lorem ipsum", "", "lorem ipsum")]
#[case::transaction_comment_multiword_space("; lorem ipsum", "", "lorem ipsum")]
fn test_parse_transaction_comment(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_comment: &str,
) {
    let mut input = input;
    assert_eq!(
        parse_transaction_comment(&mut input).unwrap(),
        expected_comment
    );
    assert_eq!(input, expected_remaining);
}

#[test]
fn test_parse_line_comment_as_transaction_comment() {
    assert_eq!(
        parse_transaction_comment(&mut "# comment").unwrap_err(),
        ErrMode::Backtrack(ContextError::new()) // TODO: errors
    );
}
