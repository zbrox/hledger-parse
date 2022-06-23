use std::path::PathBuf;

use crate::{
    account::types::Account, commodity::types::Commodity, price::types::Price,
    transaction::types::Transaction, HLParserError,
};

use super::parsers::parse_journal;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Ignore,
    Transaction(Transaction),
    Included(Vec<Value>),
    Price(Price),
    Account(Account),
    Commodity(Commodity),
}

#[derive(PartialEq, Debug)]
pub struct Journal {
    transactions: Vec<Transaction>,
    accounts: Vec<String>,
    prices: Vec<Price>,
    commodities: Vec<Commodity>,
}

impl TryFrom<PathBuf> for Journal {
    type Error = HLParserError;

    fn try_from(journal_path: PathBuf) -> Result<Self, HLParserError> {
        let base_path = journal_path.parent().map(|v| v.to_owned());
        let journal_contents = std::fs::read_to_string(journal_path)?;
        let journal = parse_journal(&journal_contents, base_path).map_err(|e| match e {
            HLParserError::Parse(i, ek) => HLParserError::Parse(i, ek),
            HLParserError::Validation(desc) => HLParserError::Validation(desc),
            HLParserError::IO(e) => HLParserError::IO(e),
            HLParserError::IncludePath(path) => HLParserError::IncludePath(path),
            HLParserError::Extract(v) => HLParserError::Extract(v),
        })?;

        Ok(journal)
    }
}

impl Journal {
    pub fn new(
        transactions: Vec<Transaction>,
        accounts: Vec<Account>,
        prices: Vec<Price>,
        commodities: Vec<Commodity>,
    ) -> Journal {
        Journal {
            transactions,
            accounts,
            prices,
            commodities,
        }
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    pub fn accounts(&self) -> Vec<Account> {
        self.accounts.clone()
    }

    pub fn prices(&self) -> Vec<Price> {
        self.prices.clone()
    }

    pub fn commodities(&self) -> Vec<Commodity> {
        self.commodities.clone()
    }

    pub fn payees(&self) -> Vec<String> {
        let mut tx_payees: Vec<String> = self
            .transactions
            .iter()
            .filter_map(|t| t.description.payee.clone())
            .collect();
        tx_payees.sort();
        let mut unique_payees: Vec<String> = vec![];

        tx_payees.iter().for_each(|p| {
            if !unique_payees.contains(p) {
                unique_payees.push(p.clone());
            }
        });

        unique_payees
    }
}