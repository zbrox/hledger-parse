use chrono::{Datelike, NaiveDate};
use winnow::{
    ascii::dec_int,
    combinator::{alt, opt, preceded, separated},
    error::{ContextError, ErrMode},
    PResult, Parser,
};

fn parse_date_components<'s>(
    separator: char,
) -> impl FnMut(&mut &'s str) -> PResult<(Option<i32>, u32, u32)> {
    move |input: &mut &'s str| {
        let comps: Vec<i32> = separated(0.., dec_int::<_, i32, _>, separator).parse_next(input)?;
        match comps.len() {
            3 => Ok((Some(comps[0]), comps[1] as u32, comps[2] as u32)),
            2 => Ok((None, comps[0] as u32, comps[1] as u32)),
            _ => Err(ErrMode::Backtrack(ContextError::new())), // TODO: better error
        }
    }
}

// TODO: this is not great, do something about it
fn parse_separator_date<'s>(
    separator: char,
) -> impl FnMut(&mut &'s str) -> PResult<(NaiveDate, Option<NaiveDate>)> {
    move |i: &mut &'s str| {
        let (primary_date_components, secondary_date_components) = (
            parse_date_components(separator),
            opt(preceded('=', parse_date_components(separator))),
        )
            .parse_next(i)?;

        // TODO: need better errors
        let (y, m, d) = match primary_date_components {
            (Some(y), m, d) => (y, m, d),
            _ => return Err(ErrMode::Backtrack(ContextError::new())), // TODO: better error
        };

        let primary_date = match NaiveDate::from_ymd_opt(y, m, d) {
            Some(date) => date,
            None => {
                return Err(ErrMode::Backtrack(ContextError::new())); // TODO: errors
                                                                     // TODO: better error
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
                None => return Err(ErrMode::Backtrack(ContextError::new())), // TODO: better error
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
