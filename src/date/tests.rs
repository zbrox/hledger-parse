use chrono::NaiveDate;
use rstest::rstest;

use crate::date::parsers::parse_date;

#[rstest]
#[case::date_dash("2020-01-01", "", NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(), None)]
#[case::date_slash("2020/01/01", "", NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(), None)]
#[case::date_dot("2020.01.01", "", NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(), None)]
#[case::date_dash_secondary(
    "2020-01-01=2020-01-02",
    "",
    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
    Some(NaiveDate::from_ymd_opt(2020, 1, 2).unwrap())
)]
#[case::date_slash_secondary(
    "2020/01/01=2020/01/02",
    "",
    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
    Some(NaiveDate::from_ymd_opt(2020, 1, 2).unwrap())
)]
#[case::date_dot_secondary(
    "2020.01.01=2020.01.02",
    "",
    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
    Some(NaiveDate::from_ymd_opt(2020, 1, 2).unwrap())
)]
#[case::smart_secondary_date(
    "2020-01-01=12-5",
    "",
    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
    Some(NaiveDate::from_ymd_opt(2020, 12, 5).unwrap())
)]
fn test_parse_date_dash(
    #[case] input: &str,
    #[case] expected_remaining: &str,
    #[case] expected_date: NaiveDate,
    #[case] expected_secondary_date: Option<NaiveDate>,
) {
    let mut input = input;
    assert_eq!(
        parse_date(&mut input).unwrap(),
        (expected_date, expected_secondary_date)
    );
    assert_eq!(input, expected_remaining);
}

#[rstest]
#[case::month_greater_than_12(&mut "2020.13.01")]
#[case::month_zero("2020.00.01")]
#[case::day_greater_than_31("2020.01.32")]
#[case::day_zero("2020.01.00")]
#[case::day_29_non_leap_year("2021.02.29")]
#[case::mixed_separator("2021/02.29")]
fn test_parse_date_invalid(#[case] input: &str) {
    let mut input = input;
    assert!(parse_date(&mut input).is_err(),);
}
