use winnow::{
    ascii::{line_ending, space0, till_line_ending},
    combinator::{alt, eof, opt, preceded, repeat, separated, terminated},
    error::{ErrMode, FromExternalError as _, StrContext},
    token::take,
    PResult, Parser,
};

use crate::{
    code::parsers::parse_code,
    comment::parsers::parse_transaction_comment,
    date::parsers::parse_date,
    description::parsers::parse_description,
    posting::parsers::parse_posting,
    status::parsers::parse_status,
    tag::{parsers::parse_tag, types::Tag},
    utils::find_space_before_char,
};

use super::types::Transaction;

pub(super) fn parse_comments_tags<'s>(input: &mut &'s str) -> PResult<(&'s str, Vec<Tag>)> {
    let mut line = till_line_ending.parse_next(input)?;
    let comment = match find_space_before_char(line, ':') {
        Some(pos) => take(pos + 1).parse_next(&mut line)?,
        None => "",
    };

    let tags = terminated(
        separated(0.., parse_tag, terminated(',', space0)),
        alt((line_ending, eof)),
    )
    .context(StrContext::Label("tags"))
    .parse_next(&mut line)?;

    Ok((comment.trim(), tags))
}

pub fn parse_transaction(input: &mut &str) -> PResult<Transaction> {
    let (primary_date, secondary_date) = terminated(parse_date, space0).parse_next(input)?;
    let status = parse_status
        .context(StrContext::Label("transaction status"))
        .parse_next(input)?;
    let code = opt(parse_code.context(StrContext::Label("transaction code"))).parse_next(input)?;

    let (description, comment_and_tags) = terminated(
        (
            parse_description.context(StrContext::Label("transaction description")),
            opt(preceded(
                space0,
                parse_transaction_comment
                    .and_then(parse_comments_tags)
                    .context(StrContext::Label("transaction comment and tags")),
            )),
        ),
        line_ending,
    )
    .parse_next(input)?;

    let postings =
        repeat(0.., terminated(parse_posting, alt((line_ending, eof)))).parse_next(input)?;

    let transaction = Transaction {
        primary_date,
        secondary_date,
        code: code.map(str::to_string),
        status,
        description,
        tags: match comment_and_tags {
            Some((_, tags)) => tags,
            None => vec![],
        },
        postings,
    };

    transaction.validate().map_err(|e| {
        ErrMode::from_external_error(input, winnow::error::ErrorKind::Verify, e).cut()
    })?;

    Ok(transaction)
}
