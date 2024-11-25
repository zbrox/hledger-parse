use winnow::{
    ascii::{space0, space1, till_line_ending},
    combinator::{alt, delimited, empty, opt, rest, separated_pair},
    token::{literal, take_until},
    PResult, Parser,
};

use crate::{amount::parsers::parse_amount, status::parsers::parse_status, Amount};

use super::types::{Posting, PostingComplexAmount};

fn parse_posting_with_amount<'s>(input: &mut &'s str) -> PResult<PostingComplexAmount> {
    space0.parse_next(input)?;
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

    Ok(complex_amount)
}

pub(super) fn parse_balance_assertion(input: &mut &str) -> PResult<Amount> {
    let balance_assertion = delimited(
        delimited(space0, literal('='), space0),
        parse_amount,
        space0,
    )
    .parse_next(input)?;

    Ok(balance_assertion)
}

pub fn parse_posting(input: &mut &str) -> PResult<Posting> {
    let status = delimited(space1, parse_status, space0)
        .context(winnow::error::StrContext::Label(
            "error parsing posting status",
        ))
        .parse_next(input)?;

    let mut rest_of_line = alt((till_line_ending, rest)).parse_next(input)?;

    if rest_of_line.contains("  ") {
        let account_name = take_until(1.., "  ")
            .context(winnow::error::StrContext::Label(
                "error parsing account name in posting with amount",
            ))
            .parse_next(&mut rest_of_line)?;

        let complex_amount = alt((
            parse_posting_with_amount,
            empty.value(PostingComplexAmount::default()),
        ))
        .context(winnow::error::StrContext::Label(
            "error parsing posting complex amount",
        ))
        .parse_next(&mut rest_of_line)?;
        let balance_assertion = opt(parse_balance_assertion)
            .context(winnow::error::StrContext::Label(
                "error parsing posting balance assertion",
            ))
            .parse_next(&mut rest_of_line)?;

        Ok(Posting {
            status,
            account: account_name.trim().into(),
            amount: complex_amount.amount,
            unit_price: complex_amount.unit_price,
            total_price: complex_amount.total_price,
            balance_assertion: balance_assertion,
        })
    } else {
        Ok(Posting {
            status,
            account: rest_of_line.trim().into(),
            amount: None,
            unit_price: None,
            total_price: None,
            balance_assertion: None,
        })
    }
}
