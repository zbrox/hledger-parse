use rstest::rstest;

use crate::{
    comment::parsers::{parse_line_comment, parse_transaction_comment},
    HLParserError,
};

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
    assert_eq!(
        parse_line_comment(input).unwrap(),
        (expected_remaining, expected_comment)
    );
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
    assert_eq!(
        parse_transaction_comment(input).unwrap(),
        (expected_remaining, expected_comment)
    );
}

#[test]
fn test_parse_line_comment_as_transaction_comment() {
    assert_eq!(
        parse_transaction_comment("# comment")
            .unwrap_err()
            .to_string(),
        nom::Err::Error(HLParserError::Parse(
            "# comment".to_owned(),
            nom::error::ErrorKind::Char
        ))
        .to_string()
    );
}
