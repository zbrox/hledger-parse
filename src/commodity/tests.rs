use rstest::rstest;

use crate::commodity::types::Commodity;

use super::parsers::parse_commodity_directive;

#[rstest]
#[case("commodity $1000.00", "", "$", "$1000.00")]
#[case("commodity $ 1000.00", "", "$", "$ 1000.00")]
#[case("commodity 1000.00USD", "", "USD", "1000.00USD")]
#[case("commodity 1000.00 USD", "", "USD", "1000.00 USD")]
fn test_parse_commodity_directive_single_line(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_currency: &str,
    #[case] expected_format: &str,
) {
    let mut input = input;
    assert_eq!(
        parse_commodity_directive(&mut input).unwrap(),
        Commodity {
            name: expected_currency.to_string(),
            format: Some(expected_format.to_string()),
        }
    );
    assert_eq!(input, expected_remaining);
}

#[test]
fn test_parse_commodity_directive_multi_line() {
    let mut input = r#"commodity USD
    format 1000.00USD
"#;
    assert_eq!(
        parse_commodity_directive(&mut input).unwrap(),
        Commodity {
            name: "USD".to_string(),
            format: Some("1000.00USD".to_string()),
        }
    );
    assert_eq!(input, "");
}

#[test]
fn test_parse_commodity_name_only() {
    let mut input = r#"commodity INR"#;
    assert_eq!(
        parse_commodity_directive(&mut input).unwrap(),
        Commodity {
            name: "INR".to_string(),
            format: None
        }
    );
    assert_eq!(input, "");
}
