use std::{path::PathBuf};

use hledger_parse::{types::{Journal, HLParserError}};

fn main() -> Result<(), HLParserError<String>> {
    let journal_path = PathBuf::from("./examples/simple.journal");

    let journal: Journal = journal_path.try_into()?;

    println!("Transactions: {}", journal.transactions.len());

    Ok(())
}