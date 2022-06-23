use std::fmt::Display;

use rust_decimal::Decimal;

#[derive(PartialEq)]
pub enum AmountSign {
    Plus,
    Minus,
}

/// Stores information for an amount
///
/// Defaults to displaying amounts in the format `<VALUE> <CURRENCY>`, using a dot as the decimal separator.
///
/// # Example:
///
/// ```
/// use rust_decimal_macros::dec;
/// use hledger_parse::Amount;
///
/// let amount = Amount { currency: "EUR".to_string(), value: dec!(19.99) };
/// assert_eq!("19.99 EUR", format!("{}", amount));
/// ```
#[derive(PartialEq, Debug, Clone)]
pub struct Amount {
    pub currency: String,
    pub value: Decimal,
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}
