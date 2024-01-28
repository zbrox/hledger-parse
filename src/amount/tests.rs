use rstest::rstest;
use rust_decimal_macros::dec;

use crate::amount::{
    parsers::{parse_amount, parse_currency_string},
    types::Amount,
};

use super::parsers::parse_money_amount;

#[rstest]
#[case::simple("$100", "", "$", dec!(100))]
#[case::simple_negative_after_curr("$-100", "", "$", dec!(-100))]
#[case::simple_negative_before_curr("-$100", "", "$", dec!(-100))]
#[case::simple_single_space("$ 100", "", "$", dec!(100))]
#[case::simple_negative_after_curr_with_space("$ -100", "", "$", dec!(-100))]
#[case::simple_multiple_spaces("$  100", "", "$", dec!(100))]
#[case::simple_negative_multiple_spaces("$  -100", "", "$", dec!(-100))]
#[case::complex_curr("\"silver coins\" 100", "", "silver coins", dec!(100))]
#[case::complex_curr_negative("\"silver coins\"- 100", "", "silver coins", dec!(-100))]
#[case::complex_curr_negative_prefix("-\"silver coins\" 100", "", "silver coins", dec!(-100))]
#[case::complex_curr_multiple_spaces("\"silver coins\"   100", "", "silver coins", dec!(100))]
#[case::complex_curr_negative_before_amount_multiple_spaces("\"silver coins\"   -100", "", "silver coins", dec!(-100))]
#[case::suffix_curr("100EUR", "", "EUR", dec!(100))]
#[case::suffix_curr_negative("-100EUR", "", "EUR", dec!(-100))]
#[case::suffix_curr_negative_with_space("- 100EUR", "", "EUR", dec!(-100))]
#[case::suffix_curr_negative_multiple_spaces("-   100EUR", "", "EUR", dec!(-100))]
#[case::suffix_curr_space("100 EUR", "", "EUR", dec!(100))]
#[case::suffix_curr_space_negative_prefix("-100 EUR", "", "EUR", dec!(-100))]
#[case::suffix_curr_space_negative_multispace_prefix("- 100 EUR", "", "EUR", dec!(-100))]
#[case::suffix_curr_multispace("100   EUR", "", "EUR", dec!(100))]
#[case::suffix_curr_multispace_negative_sufix("-100   EUR", "", "EUR", dec!(-100))]
#[case::suffix_curr_negative_prefix_space("- 100   EUR", "", "EUR", dec!(-100))]
#[case::complex_suffix("100 \"silver coins\"", "", "silver coins", dec!(100))]
#[case::complex_suffix_multispace("100   \"silver coins\"", "", "silver coins", dec!(100))]
fn test_parse_amount_currency(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_currency: &str,
    #[case] expected_value: rust_decimal::Decimal,
) {
    assert_eq!(
        parse_amount(input).unwrap(),
        (
            expected_remaining,
            Amount {
                currency: expected_currency.into(),
                value: expected_value
            }
        )
    );
}

#[rstest]
#[case::simple_int("100EUR", "EUR", dec!(100))]
#[case::double_dot("100.95EUR", "EUR", dec!(100.95))]
#[case::double_comma("100,95EUR", "EUR", dec!(100.95))]
#[case::double_small_amount("0.007EUR", "EUR", dec!(0.007))]
fn test_parse_money_amount(
    #[case] input: &str,
    #[case] expected_currency: &str,
    #[case] expected_value: rust_decimal::Decimal,
) {
    assert_eq!(
        parse_money_amount(input).unwrap(),
        (expected_currency, expected_value)
    )
}

#[rstest]
#[case::simple("$", "", "$")]
#[case::simple_space("$ ", " ", "$")]
#[case::in_quotes("\"Imaginary money\"", "", "Imaginary money")]
#[case::in_quotes_space("\"Imaginary money\" ", " ", "Imaginary money")]
#[case::iso("USD", "", "USD")]
#[case::iso_space("USD ", " ", "USD")]
fn test_parse_currency_string(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_currency: &str,
) {
    assert_eq!(
        parse_currency_string(input).unwrap(),
        (expected_remaining, expected_currency)
    );
}
