use rust_decimal::Decimal;

#[derive(PartialEq)]
pub enum AmountSign {
    Plus,
    Minus,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Amount {
    pub currency: String,
    pub value: Decimal,
}
