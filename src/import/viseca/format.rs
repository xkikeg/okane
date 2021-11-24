use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Entry represents one entry in Viseca PDF.
#[derive(Debug, PartialEq, Clone)]
pub struct Entry {
    pub line_count: usize,
    pub date: NaiveDate,
    pub effective_date: NaiveDate,
    pub payee: String,
    pub amount: Decimal,
    pub category: String,
    pub spent: Option<Amount>,
    pub exchange: Option<Exchange>,
    pub fee: Option<Fee>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Amount {
    pub value: Decimal,
    pub currency: String,
}

/// Exchange is a FX info.
#[derive(Debug, PartialEq, Clone)]
pub struct Exchange {
    pub rate: Decimal,
    pub rate_date: NaiveDate,
    /// original currency representation of spent.
    pub equivalent: Amount,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fee {
    /// fee amount and currency.
    pub percent: Decimal,
    pub amount: Amount,
}
