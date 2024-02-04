use rstest::rstest;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use winnow::error::{ContextError, ErrMode};

use crate::{amount::types::Amount, status::types::Status};

use super::{parsers::parse_posting, types::Posting};

#[rstest]
#[case::simple(" assets:cash  $100", "", Status::Unmarked, "assets:cash", "$", dec!(100))]
#[case::various_spacing(" assets:cash      $100", "", Status::Unmarked, "assets:cash", "$", dec!(100))]
#[case::with_status(" ! assets:cash  $100", "", Status::Pending, "assets:cash", "$", dec!(100))]
fn test_parse_simple_posting(
    #[case] input: &str,
    #[case] expected_rest: &str,
    #[case] expected_status: Status,
    #[case] expected_account: &str,
    #[case] expected_currency: &str,
    #[case] expected_value: Decimal,
) {
    let mut input = input;
    assert_eq!(
        parse_posting(&mut input).unwrap(),
        Posting {
            status: expected_status,
            account: expected_account.into(),
            amount: Some(Amount {
                currency: expected_currency.into(),
                value: expected_value,
            }),
            unit_price: None,
            total_price: None,
        }
    );
    assert_eq!(input, expected_rest);
}

#[test]
fn test_correct_termination_parse_posting() {
    let mut input = " assets:cash\n2008/06/01 gift\n  assets:bank:checking  $1";
    assert_eq!(
        parse_posting(&mut input).unwrap(),
        Posting {
            status: Status::Unmarked,
            account: "assets:cash".into(),
            amount: None,
            unit_price: None,
            total_price: None,
        }
    );
    assert_eq!(input, "\n2008/06/01 gift\n  assets:bank:checking  $1");
}

#[test]
fn test_parse_posting_without_amount() {
    let mut input = " assets:cash";
    assert_eq!(
        parse_posting(&mut input).unwrap(),
        Posting {
            status: Status::Unmarked,
            account: "assets:cash".into(),
            amount: None,
            unit_price: None,
            total_price: None,
        }
    );
    assert_eq!(input, "");
}

#[test]
fn test_parse_posting_no_starting_space() {
    assert_eq!(
        parse_posting(&mut "assets:cash").unwrap_err(),
        ErrMode::Backtrack(ContextError::new()) // TODO: errors
    )
}

#[test]
fn test_parse_posting_with_unit_price() {
    assert_eq!(
        parse_posting(&mut " ! assets:cash  $100 @ EUR0.94").unwrap(),
        Posting {
            status: Status::Pending,
            account: "assets:cash".into(),
            amount: Some(Amount {
                currency: "$".into(),
                value: dec!(100)
            }),
            unit_price: Some(Amount {
                currency: "EUR".into(),
                value: dec!(0.94),
            }),
            total_price: None,
        }
    )
}

#[test]
fn test_parse_posting_with_total_price() {
    assert_eq!(
        parse_posting(&mut " ! assets:cash  $100 @@ €93,89").unwrap(),
        Posting {
            status: Status::Pending,
            account: "assets:cash".into(),
            amount: Some(Amount {
                currency: "$".into(),
                value: dec!(100)
            }),
            unit_price: None,
            total_price: Some(Amount {
                currency: "€".into(),
                value: dec!(93.89),
            }),
        }
    )
}
