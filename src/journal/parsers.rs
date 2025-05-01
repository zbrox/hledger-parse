use std::{path::PathBuf, str::FromStr};

use winnow::{
    ascii::{line_ending, space0, space1, till_line_ending},
    combinator::{alt, eof, preceded, repeat_till, terminated},
    error::{ContextError, StrContext},
    Parser, Result as PResult,
};

use crate::{
    account::{parsers::parse_account_directive, types::Account},
    comment::parsers::parse_line_comment,
    commodity::{parsers::parse_commodity_directive, types::Commodity},
    price::{parsers::parse_price, types::Price},
    transaction::{parsers::parse_transaction, types::Transaction},
    HLParserError,
};

use super::types::{Journal, Value};

fn parse_include_statement(input: &mut &str) -> PResult<PathBuf> {
    let path = preceded(("include", space1), alt((till_line_ending, eof))).parse_next(input)?;
    let path = PathBuf::from_str(path).map_err(|_| ContextError::new())?; // TODO: better error
    Ok(path)
}

pub(super) fn parse_comment_value(input: &mut &str) -> PResult<Value> {
    terminated(parse_line_comment, alt((line_ending, eof)))
        .value(Value::Ignore)
        .parse_next(input)
}

pub(super) fn parse_empty_line(input: &mut &str) -> PResult<Value> {
    terminated(space0, alt((line_ending, eof)))
        .value(Value::Ignore)
        .parse_next(input)
}

pub fn read_journal_from_path(path: PathBuf) -> Result<Vec<Value>, HLParserError> {
    let contents = std::fs::read_to_string(&path).map_err(|e| HLParserError::IO(e.to_string()))?;
    let mut input = &contents[..];
    let values = parse_journal_contents(&mut input, path)?;
    Ok(values)
}

fn parse_journal_contents(
    input: &mut &str,
    base_path: PathBuf,
) -> Result<Vec<Value>, HLParserError> {
    let res = repeat_till(
        0..,
        alt((
            parse_empty_line.context(StrContext::Label("parsing empty lines")),
            parse_comment_value.context(StrContext::Label("parsing line comment")),
            parse_account_directive
                .map(|v| Value::Account(v.into()))
                .context(StrContext::Label("parsing account directive")),
            parse_commodity_directive
                .map(Value::Commodity)
                .context(StrContext::Label("parsing commodity directive")),
            parse_price
                .map(Value::Price)
                .context(StrContext::Label("parsing pricing entry")),
            parse_include_statement
                .try_map(|v| {
                    let values = match read_journal_from_path(base_path.join(v)) {
                        Ok(values) => values,
                        Err(e) => return Err(e),
                    };
                    Ok(Value::Included(values))
                })
                .context(StrContext::Label("parsing included journal")),
            parse_transaction
                .map(Value::Transaction)
                .context(StrContext::Label("parsing transaction")),
        )),
        eof,
    )
    .map(|(v, _)| v)
    .parse(input)
    .map_err(|e| HLParserError::Parse(e.to_string()))?;

    Ok(res)
}

pub(super) fn flatten_values(values: Vec<Value>) -> Vec<Value> {
    values
        .into_iter()
        .flat_map(|v| match v {
            Value::Included(contents) => flatten_values(contents),
            _ => vec![v],
        })
        .collect()
}

pub fn parse_journal(
    input: &mut &str,
    base_path: Option<PathBuf>,
) -> Result<Journal, HLParserError> {
    let values = parse_journal_contents(
        input,
        base_path.unwrap_or(std::env::current_dir().map_err(|e| HLParserError::IO(e.to_string()))?),
    )?;
    let values = flatten_values(values);

    Ok(Journal::new(
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Transaction>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Account>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Price>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Commodity>>(),
    ))
}
