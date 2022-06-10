use chrono::{Datelike, NaiveDate};
use nom::{
    branch::alt,
    character::complete::{char, i32},
    combinator::{map_res, opt},
    multi::separated_list1,
    sequence::{preceded, tuple},
};

use crate::{HLParserError, HLParserIResult};

fn parse_date_components(
    separator: char,
) -> impl FnMut(&str) -> HLParserIResult<&str, (Option<i32>, u32, u32)> {
    move |input: &str| {
        map_res(
            separated_list1(char(separator), i32),
            |comps: Vec<i32>| match comps.len() {
                3 => Ok((Some(comps[0]), comps[1] as u32, comps[2] as u32)),
                2 => Ok((None, comps[0] as u32, comps[1] as u32)),
                _ => Err(nom::Err::Error(HLParserError::Parse(
                    input.to_string(),
                    nom::error::ErrorKind::Tag,
                ))),
            },
        )(input)
    }
}

// TODO: this is not great, do something about it
fn parse_separator_date(
    separator: char,
) -> impl FnMut(&str) -> HLParserIResult<&str, (NaiveDate, Option<NaiveDate>)> {
    move |i: &str| {
        let (tail, (primary_date_components, secondary_date_components)) = tuple((
            parse_date_components(separator),
            opt(preceded(char('='), parse_date_components(separator))),
        ))(i)?;

        // TODO: need better errors
        let (y, m, d) = match primary_date_components {
            (Some(y), m, d) => (y, m, d),
            _ => {
                return Err(nom::Err::Error(HLParserError::Parse(
                    i.to_string(),
                    nom::error::ErrorKind::Tag,
                )))
            }
        };

        let primary_date = match NaiveDate::from_ymd_opt(y, m as u32, d as u32) {
            Some(date) => date,
            None => {
                return Err(nom::Err::Error(HLParserError::Parse(
                    i.to_string(),
                    nom::error::ErrorKind::Tag,
                )))
            }
        };

        let secondary_date_components = match secondary_date_components {
            Some(secondary_date_components) => match secondary_date_components {
                (Some(sec_y), sec_m, sec_d) => Some((sec_y, sec_m, sec_d)),
                (None, sec_m, sec_d) => Some((primary_date.year(), sec_m, sec_d)),
            },
            None => None,
        };

        let secondary_date = match secondary_date_components {
            Some((y, m, d)) => match NaiveDate::from_ymd_opt(y, m as u32, d as u32) {
                Some(date) => Some(date),
                None => {
                    return Err(nom::Err::Error(HLParserError::Parse(
                        i.to_string(),
                        nom::error::ErrorKind::Tag,
                    )))
                }
            },
            None => None,
        };

        Ok((tail, (primary_date, secondary_date)))
    }
}

pub fn parse_date(input: &str) -> HLParserIResult<&str, (NaiveDate, Option<NaiveDate>)> {
    alt((
        parse_separator_date('-'),
        parse_separator_date('/'),
        parse_separator_date('.'),
    ))(input)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use nom::error::ErrorKind;

    use crate::{parsers::dates::parse_date, HLParserError};

    #[test]
    fn test_parse_date_dash() {
        assert_eq!(
            parse_date("2020-01-01").unwrap(),
            ("", (NaiveDate::from_ymd(2020, 1, 1), None))
        );
    }

    #[test]
    fn test_parse_date_slash() {
        assert_eq!(
            parse_date("2020/01/01").unwrap(),
            ("", (NaiveDate::from_ymd(2020, 1, 1), None))
        );
    }

    #[test]
    fn test_parse_date_dot() {
        assert_eq!(
            parse_date("2020.01.01").unwrap(),
            ("", (NaiveDate::from_ymd(2020, 1, 1), None))
        );
    }

    #[test]
    fn test_parse_date_invalid_month() {
        assert_eq!(
            parse_date("2020.13.01").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "2020.13.01".to_string(),
                ErrorKind::Tag
            ))
            .to_string()
        );
        assert_eq!(
            parse_date("2020.00.01").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "2020.00.01".to_string(),
                ErrorKind::Tag
            ))
            .to_string()
        );
    }

    #[test]
    fn test_parse_date_invalid_day() {
        assert_eq!(
            parse_date("2021.02.29").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "2021.02.29".to_string(),
                ErrorKind::Tag
            ))
            .to_string()
        );
        assert_eq!(
            parse_date("2021.02.60").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "2021.02.60".to_string(),
                ErrorKind::Tag
            ))
            .to_string()
        );
    }

    #[test]
    fn test_parse_date_mix_separator() {
        assert_eq!(
            parse_date("2021/02.29").unwrap_err().to_string(),
            nom::Err::Error(HLParserError::Parse(
                "2021/02.29".to_string(),
                ErrorKind::MapRes
            ))
            .to_string()
        );
    }

    #[test]
    fn test_parse_date_secondary_dash() {
        assert_eq!(
            parse_date("2021-01-01=2021-01-05").unwrap(),
            (
                "",
                (
                    NaiveDate::from_ymd(2021, 1, 1),
                    Some(NaiveDate::from_ymd(2021, 1, 5))
                )
            ),
        )
    }

    #[test]
    fn test_parse_date_secondary_slash() {
        assert_eq!(
            parse_date("2021/01/01=2021/01/05").unwrap(),
            (
                "",
                (
                    NaiveDate::from_ymd(2021, 1, 1),
                    Some(NaiveDate::from_ymd(2021, 1, 5))
                )
            ),
        )
    }

    #[test]
    fn test_parse_date_secondary_dot() {
        assert_eq!(
            parse_date("2021.01.01=2021.01.05").unwrap(),
            (
                "",
                (
                    NaiveDate::from_ymd(2021, 1, 1),
                    Some(NaiveDate::from_ymd(2021, 1, 5))
                )
            ),
        )
    }

    #[test]
    fn test_parse_date_secondary_smart_date() {
        assert_eq!(
            parse_date("2021-01-01=01-05").unwrap(),
            (
                "",
                (
                    NaiveDate::from_ymd(2021, 1, 1),
                    Some(NaiveDate::from_ymd(2021, 1, 5))
                )
            ),
        );
        assert_eq!(
            parse_date("2021-01-01=12-5").unwrap(),
            (
                "",
                (
                    NaiveDate::from_ymd(2021, 1, 1),
                    Some(NaiveDate::from_ymd(2021, 12, 5))
                )
            ),
        );
    }
}
