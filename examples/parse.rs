use std::path::PathBuf;

use hledger_parse::{parse_journal, HLParserError, Journal};

fn main() -> Result<(), HLParserError<String>> {
    let journal_path = PathBuf::from("./examples/10000x1000x10.journal");
    let base_path = journal_path.parent().map(|v| v.to_owned());
    let contents = std::fs::read_to_string(&journal_path)
        .map_err(|e| HLParserError::IO(format!("Error reading journal file: {}", e.to_string())))?;
    let mut input = contents.as_str();

    let journal: Journal = parse_journal(&mut input, base_path)?;

    println!("Transactions: {}", journal.transactions().len());
    println!("{}", journal);

    Ok(())
}
