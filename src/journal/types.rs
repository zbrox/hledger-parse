use std::fmt::Display;

use crate::{
    account::types::Account, commodity::types::Commodity, price::types::Price,
    transaction::types::Transaction, HLParserError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Ignore,
    Transaction(Transaction),
    Included(Vec<Value>),
    Price(Price),
    Account(Account),
    Commodity(Commodity),
}

/// A journal is a collection of transactions, accounts, prices, and commodities
#[derive(PartialEq, Eq, Debug)]
pub struct Journal {
    transactions: Vec<Transaction>,
    accounts: Vec<Account>,
    prices: Vec<Price>,
    commodities: Vec<Commodity>,
}

// impl<'s> TryFrom<&'s PathBuf> for Journal {
//     type Error = HLParserError<&'s str>;

//     fn try_from(journal_path: &'s PathBuf) -> Result<Self, Self::Error> {
//         let base_path = journal_path.parent().map(|v| v.to_owned());

//         let journal_contents = std::fs::read_to_string(journal_path).map_err(|e| {
//             HLParserError::IO(format!("Error reading journal file: {}", e.to_string()))
//         })?;
//         let mut journal_contents = journal_contents.as_str();
//         let journal = parse_journal(&mut journal_contents, base_path)?;

//         Ok(journal)
//     }
// }

impl Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for a in &self.accounts {
            writeln!(f, "account {}", a)?;
        }
        for c in &self.commodities {
            writeln!(f, "{}", c)?;
        }
        for p in &self.prices {
            writeln!(f, "{}", p)?;
        }
        for t in &self.transactions {
            writeln!(f, "{}", t)?;
        }

        Ok(())
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

    pub fn validate_accounts(&self) -> Result<(), HLParserError<&str>> {
        let undefined_accounts: Vec<Account> = self
            .transactions
            .iter()
            .flat_map(|t| t.postings.clone())
            .map(|p| p.account)
            .filter(|a| !self.accounts.contains(a))
            .collect();
        if !undefined_accounts.is_empty() {
            return Err(HLParserError::Validation(
                crate::ValidationError::UndefinedAccounts(
                    undefined_accounts
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                ),
            ));
        }
        Ok(())
    }
}
