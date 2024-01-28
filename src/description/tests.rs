use rstest::rstest;

use super::{parsers::parse_description, types::Description};

#[rstest]
#[case::simple("some description", "", None, Some("some description".to_string()))]
#[case::payee_and_note(
    "Acme | some description",
    "",
    Some("Acme".to_string()),
    Some("some description".to_string())
)]
#[case::payee_and_note_irregular_spacing(
    "Acme| some description",
    "",
    Some("Acme".to_string()),
    Some("some description".to_string())
)]
#[case::payee_and_note_irregular_spacing_2(
    "Acme |some description",
    "",
    Some("Acme".to_string()),
    Some("some description".to_string())
)]
#[case::payee_and_note_no_space(
    "Acme|some description",
    "",
    Some("Acme".to_string()),
    Some("some description".to_string())
)]
#[case::empty_space(" ", "", None, None)]
#[case::empty("", "", None, None)]
fn test_parse_description(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_payee: Option<String>,
    #[case] expected_note: Option<String>,
) {
    assert_eq!(
        parse_description(input).unwrap(),
        (
            expected_remaining,
            Description {
                payee: expected_payee,
                note: expected_note,
            }
        )
    )
}
