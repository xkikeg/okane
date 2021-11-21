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
    pub exchange: Option<Exchange>,
}

/// Exchange is a FX info.
#[derive(Debug, PartialEq, Clone)]
pub struct Exchange {
    pub rate: Decimal,
    pub rate_date: NaiveDate,
    /// exchanged currency
    pub exchanged_currency: String,
    pub exchanged_amount: Decimal,
    /// spent value in transformed currency.
    /// The Entry.amount is deducted value.
    pub spent_currency: String,
    pub spent_amount: Decimal,
    /// fee amount and currency.
    pub fee_percent: Decimal,
    pub fee_currency: String,
    pub fee_amount: Decimal,
}
