use chrono::NaiveDate;
use nom::error::ErrorKind;
use rstest::rstest;

use crate::{date::parsers::parse_date, HLParserError};

#[rstest]
#[case::date_dash("2020-01-01", "", NaiveDate::from_ymd(2020, 1, 1), None)]
#[case::date_slash("2020/01/01", "", NaiveDate::from_ymd(2020, 1, 1), None)]
#[case::date_dot("2020.01.01", "", NaiveDate::from_ymd(2020, 1, 1), None)]
#[case::date_dash_secondary(
    "2020-01-01=2020-01-02",
    "",
    NaiveDate::from_ymd(2020, 1, 1),
    Some(NaiveDate::from_ymd(2020, 1, 2))
)]
#[case::date_slash_secondary(
    "2020/01/01=2020/01/02",
    "",
    NaiveDate::from_ymd(2020, 1, 1),
    Some(NaiveDate::from_ymd(2020, 1, 2))
)]
#[case::date_dot_secondary(
    "2020.01.01=2020.01.02",
    "",
    NaiveDate::from_ymd(2020, 1, 1),
    Some(NaiveDate::from_ymd(2020, 1, 2))
)]
#[case::smart_secondary_date(
    "2020-01-01=12-5",
    "",
    NaiveDate::from_ymd(2020, 1, 1),
    Some(NaiveDate::from_ymd(2020, 12, 5))
)]
fn test_parse_date_dash(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_date: NaiveDate,
    #[case] expected_secondary_date: Option<NaiveDate>,
) {
    assert_eq!(
        parse_date(input).unwrap(),
        (expected_remaining, (expected_date, expected_secondary_date))
    );
}

#[rstest]
#[case::month_greater_than_12("2020.13.01", "2020.13.01", ErrorKind::Tag)]
#[case::month_zero("2020.00.01", "2020.00.01", ErrorKind::Tag)]
#[case::day_greater_than_31("2020.01.32", "2020.01.32", ErrorKind::Tag)]
#[case::day_zero("2020.01.00", "2020.01.00", ErrorKind::Tag)]
#[case::day_29_non_leap_year("2021.02.29", "2021.02.29", ErrorKind::Tag)]
#[case::mixed_separator("2021/02.29", "2021/02.29", ErrorKind::MapRes)]
fn test_parse_date_invalid(
    #[case] input: &str,
    #[case] expected_error: &str,
    #[case] expected_error_kind: ErrorKind,
) {
    assert_eq!(
        parse_date(input).unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            expected_error.to_string(),
            expected_error_kind,
        ))
        .to_string()
    );
}
