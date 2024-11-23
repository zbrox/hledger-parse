use chrono::{Datelike, NaiveDate};
use winnow::{
    combinator::{alt, opt, preceded, terminated},
    error::{ErrMode, FromExternalError as _, StrContext},
    PResult, Parser,
};

use crate::{
    utils::{deci32_leading_zeros, decu32_leading_zeros},
    ValidationError,
};

fn parse_date_components<'s>(
    separator: char,
) -> impl FnMut(&mut &'s str) -> PResult<(Option<i32>, u32, u32)> {
    move |input: &mut &'s str| {
        let first_part = terminated(deci32_leading_zeros, separator).parse_next(input)?;
        let second_part = decu32_leading_zeros.parse_next(input)?;
        let third_part = opt(preceded(separator, decu32_leading_zeros)).parse_next(input)?;

        match third_part {
            Some(third_part) => Ok((Some(first_part), second_part, third_part)),
            None => Ok((None, first_part as u32, second_part)),
        }
    }
}

// TODO: this is not great, do something about it
fn parse_separator_date<'s>(
    separator: char,
) -> impl FnMut(&mut &'s str) -> PResult<(NaiveDate, Option<NaiveDate>)> {
    move |i: &mut &'s str| {
        let (primary_date_components, secondary_date_components) = (
            parse_date_components(separator)
                .context(StrContext::Label("error parsing primary date components")),
            opt(preceded('=', parse_date_components(separator)))
                .context(StrContext::Label("error parsing secondary date components")),
        )
            .parse_next(i)?;

        let (y, m, d) = match primary_date_components {
            (Some(y), m, d) => (y, m, d),
            _ => {
                return Err(ErrMode::from_external_error(
                    i,
                    winnow::error::ErrorKind::Verify,
                    ValidationError::InvalidDateComponents(
                        primary_date_components.0,
                        primary_date_components.1,
                        primary_date_components.2,
                    ),
                )
                .cut())
            }
        };

        let primary_date = match NaiveDate::from_ymd_opt(y, m, d) {
            Some(date) => date,
            None => {
                return Err(ErrMode::from_external_error(
                    i,
                    winnow::error::ErrorKind::Verify,
                    ValidationError::InvalidDateComponents(Some(y), m, d),
                )
                .cut());
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
            Some((y, m, d)) => match NaiveDate::from_ymd_opt(y, m, d) {
                Some(date) => Some(date),
                None => {
                    return Err(ErrMode::from_external_error(
                        i,
                        winnow::error::ErrorKind::Verify,
                        ValidationError::InvalidDateComponents(Some(y), m, d),
                    )
                    .cut())
                }
            },
            None => None,
        };

        Ok((primary_date, secondary_date))
    }
}

pub fn parse_date(input: &mut &str) -> PResult<(NaiveDate, Option<NaiveDate>)> {
    alt((
        parse_separator_date('-'),
        parse_separator_date('/'),
        parse_separator_date('.'),
    ))
    .parse_next(input)
}
