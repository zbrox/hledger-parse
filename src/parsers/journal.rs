use nom::{
    branch::alt,
    character::complete::{line_ending, space0},
    combinator::{eof, map, value},
    multi::many_till,
    sequence::terminated,
    IResult,
};

use crate::types::{Journal, Transaction};

use super::{comments::parse_line_comment, transactions::parse_transaction};

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Ignore,
    Transaction(Transaction),
}

fn parse_comment_value(input: &str) -> IResult<&str, Value> {
    value(
        Value::Ignore,
        terminated(parse_line_comment, alt((line_ending, eof))),
    )(input)
}

fn parse_empty_line(input: &str) -> IResult<&str, Value> {
    value(Value::Ignore, terminated(space0, alt((line_ending, eof))))(input)
}

pub fn parse_journal(input: &str) -> IResult<&str, Journal> {
    let (tail, (values, _)) = many_till(
        alt((
            map(parse_transaction, Value::Transaction),
            parse_comment_value,
            parse_empty_line,
        )),
        eof,
    )(input)?;

    Ok((
        tail,
        Journal {
            transactions: values
                .into_iter()
                .map(|v| match v {
                    Value::Transaction(t) => Some(t),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<Transaction>>(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::parsers::journal::{parse_comment_value, parse_empty_line, Value};

    use super::parse_journal;

    #[test]
    fn test_parse_comment_value() {
        assert_eq!(
            parse_comment_value("; A sample journal file\n"),
            Ok(("", Value::Ignore))
        );
        assert_eq!(parse_comment_value(";\n"), Ok(("", Value::Ignore)));
        assert_eq!(parse_comment_value(";"), Ok(("", Value::Ignore)));
    }

    #[test]
    fn test_parse_empty_line() {
        assert_eq!(parse_empty_line("\n"), Ok(("", Value::Ignore)));
        assert_eq!(parse_empty_line("   \n"), Ok(("", Value::Ignore)));
    }

    #[test]
    fn test_parse_journal_simple() {
        // from https://github.com/simonmichael/hledger/blob/e9c19e12ef62d46f57d3cbbd6814dbcf04bbc508/examples/sample.journal
        let input = r#"; A sample journal file.
;
; Sets up this account tree:
; assets
;   bank
;     checking
;     saving
;   cash
; expenses
;   food
;   supplies
; income
;   gifts
;   salary
; liabilities
;   debts

; declare accounts:
; account assets:bank:checking
; account income:salary
; account income:gifts
; account assets:bank:saving
; account assets:cash
; account expenses:food
; account expenses:supplies
; account liabilities:debts

; declare commodities:
; commodity $

2008/01/01 income
    assets:bank:checking  $1
    income:salary

2008/06/01 gift
    assets:bank:checking  $1
    income:gifts

2008/06/02 save
    assets:bank:saving  $1
    assets:bank:checking

2008/06/03 * eat & shop
    expenses:food      $1
    expenses:supplies  $1
    assets:cash

2008/12/31 * pay off
    liabilities:debts  $1
    assets:bank:checking


;final comment
        "#;
        let res = parse_journal(input);
        println!("{res:?}");
        let res = res.unwrap();
        println!("Transaction count: {}", &res.1.transactions.len());
        println!(
            "Transaction 0 postings count: {}",
            res.1.transactions[0].postings.len()
        );
        assert!(parse_journal(input).is_ok());
    }
}
