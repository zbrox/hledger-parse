use std::{path::PathBuf};

use hledger_parse::{types::{Journal, ParserError}};

fn main() -> Result<(), ParserError> {
    let journal_path = PathBuf::from("./examples/simple.journal");

    let journal: Journal = journal_path.try_into()?;

    println!("Transactions: {}", journal.transactions.len());

    Ok(())
}