use std::fmt::Display;

use rust_decimal::Decimal;

#[derive(PartialEq, Eq)]
pub enum AmountSign {
    Plus,
    Minus,
}

/// Amounts used throughout
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
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Amount {
    /// The currency of the amount
    pub currency: String,
    /// The value of the amount
    pub value: Decimal,
}

impl Amount {
    pub fn negate(&self) -> Amount {
        Amount {
            currency: self.currency.clone(),
            value: -self.value,
        }
    }

    pub fn is_negative(&self) -> bool {
        self.value.is_sign_negative()
    }

    pub fn is_positive(&self) -> bool {
        self.value.is_sign_positive()
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}
