use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::import::amount::OwnedAmount;

/// Entry represents one entry in Viseca PDF.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub line_count: usize,
    pub date: NaiveDate,
    pub effective_date: NaiveDate,
    pub payee: String,
    pub amount: Decimal,
    pub category: String,
    pub spent: Option<OwnedAmount>,
    pub exchange: Option<Exchange>,
    pub fee: Option<Fee>,
}

/// Exchange is a FX info.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Exchange {
    pub rate: Decimal,
    pub rate_date: NaiveDate,
    /// original currency representation of spent.
    pub equivalent: OwnedAmount,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fee {
    /// fee amount and currency.
    pub percent: Decimal,
    pub amount: OwnedAmount,
}
