use chrono::NaiveDate;
use nom::{
    branch::alt,
    sequence::{terminated, tuple},
    IResult,
    character::complete::{char, i32, u32},
};

fn parse_separator_date(separator: char) -> impl Fn(&str) -> IResult<&str, NaiveDate> {
    move |i: &str| {
        let (tail, (year, month, day)) = tuple((
            terminated(i32, char(separator)),
            terminated(u32, char(separator)),
            u32,
        ))(i)?;

        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => Ok((tail, date)),
            None => Err(nom::Err::Error(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}

pub fn parse_simple_date(input: &str) -> IResult<&str, NaiveDate> {
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

    use crate::parsers::dates::parse_simple_date;

    #[test]
    fn test_parse_date_dash() {
        assert_eq!(
            parse_simple_date("2020-01-01"),
            Ok(("", NaiveDate::from_ymd(2020, 1, 1)))
        );
    }

    #[test]
    fn test_parse_date_slash() {
        assert_eq!(
            parse_simple_date("2020/01/01"),
            Ok(("", NaiveDate::from_ymd(2020, 1, 1)))
        );
    }

    #[test]
    fn test_parse_date_dot() {
        assert_eq!(
            parse_simple_date("2020.01.01"),
            Ok(("", NaiveDate::from_ymd(2020, 1, 1)))
        );
    }

    #[test]
    fn test_parse_date_invalid_month() {
        assert_eq!(
            parse_simple_date("2020.13.01"),
            Err(nom::Err::Error(nom::error::Error::new(
                "2020.13.01",
                ErrorKind::Tag
            )))
        );
        assert_eq!(
            parse_simple_date("2020.00.01"),
            Err(nom::Err::Error(nom::error::Error::new(
                "2020.00.01",
                ErrorKind::Tag
            )))
        );
    }

    #[test]
    fn test_parse_date_invalid_day() {
        assert_eq!(
            parse_simple_date("2021.02.29"),
            Err(nom::Err::Error(nom::error::Error::new(
                "2021.02.29",
                ErrorKind::Tag
            )))
        );
        assert_eq!(
            parse_simple_date("2021.02.60"),
            Err(nom::Err::Error(nom::error::Error::new(
                "2021.02.60",
                ErrorKind::Tag
            )))
        );
    }

    #[test]
    fn test_parse_date_mix_separator() {
        assert_eq!(
            parse_simple_date("2021/02.29"),
            Err(nom::Err::Error(nom::error::Error::new(
                "/02.29",
                ErrorKind::Char
            )))
        );
    }
}