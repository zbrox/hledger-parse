use winnow::{
    ascii::{space0, space1, till_line_ending},
    combinator::{alt, delimited, empty, opt, peek, preceded, separated_pair, terminated},
    token::take_until,
    PResult, Parser,
};

use crate::{amount::parsers::parse_amount, status::parsers::parse_status};

use super::types::{Posting, PostingComplexAmount};

fn parse_posting_with_amount<'s>(input: &mut &'s str) -> PResult<(&'s str, PostingComplexAmount)> {
    let account_name = terminated(take_until(2.., " "), peek(preceded(space1, parse_amount)))
        .verify(|s: &str| !s.contains('\n'))
        .parse_next(input)?;
    let complex_amount = alt((
        // NOTE: order of parsers is important
        separated_pair(parse_amount, delimited(space0, "@@", space0), parse_amount).map(
            |(amount, total_price)| -> PostingComplexAmount {
                PostingComplexAmount {
                    amount: Some(amount),
                    unit_price: None,
                    total_price: Some(total_price),
                }
            },
        ),
        separated_pair(parse_amount, delimited(space0, "@", space0), parse_amount).map(
            |(amount, unit_price)| -> PostingComplexAmount {
                PostingComplexAmount {
                    amount: Some(amount),
                    unit_price: Some(unit_price),
                    total_price: None,
                }
            },
        ),
        opt(parse_amount).map(|amount| -> PostingComplexAmount {
            PostingComplexAmount {
                amount,
                unit_price: None,
                total_price: None,
            }
        }),
    ))
    .parse_next(input)?;

    Ok((account_name, complex_amount))
}

fn parse_posting_without_amount<'s>(
    input: &mut &'s str,
) -> PResult<(&'s str, PostingComplexAmount)> {
    (
        till_line_ending,
        empty.value(PostingComplexAmount {
            amount: None,
            unit_price: None,
            total_price: None,
        }),
    )
        .parse_next(input)
}

pub fn parse_posting(input: &mut &str) -> PResult<Posting> {
    let (status, (account_name, complex_amount)) = (
        delimited(space1, parse_status, space0),
        alt((parse_posting_with_amount, parse_posting_without_amount)),
    )
        .parse_next(input)?;

    Ok(Posting {
        status,
        account: account_name.trim().into(),
        amount: complex_amount.amount,
        unit_price: complex_amount.unit_price,
        total_price: complex_amount.total_price,
    })
}
