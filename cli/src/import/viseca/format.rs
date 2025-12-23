use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::import::{
    amount::OwnedAmount,
    extract::{self, StrField},
};

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

impl Entry {
    pub fn as_entity<'a>(&'a self, commodity: &'a str) -> impl extract::Entity<'a> {
        Entity {
            entry: self,
            commodity,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entity<'a> {
    entry: &'a Entry,
    commodity: &'a str,
}

impl<'a> extract::Entity<'a> for Entity<'a> {
    fn str_field(&self, field: StrField) -> Option<&'a str> {
        match field {
            StrField::Camt(_) => None,
            StrField::Payee => Some(&self.entry.payee),
            StrField::Category => Some(&self.entry.category),
            StrField::Commodity => Some(self.commodity),
            StrField::SecondaryCommodity => self.entry.spent.as_ref().map(|x| x.commodity.as_str()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VisecaFormat;

impl extract::EntityFormat for VisecaFormat {
    fn name(&self) -> &'static str {
        "viseca"
    }

    fn has_camt_transaction_code(&self) -> bool {
        false
    }

    fn has_str_field(&self, field: StrField) -> bool {
        match field {
            StrField::Camt(_) => false,
            StrField::Payee => true,
            StrField::Category => true,
            StrField::Commodity => true,
            StrField::SecondaryCommodity => true,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use extract::Entity;

    #[test]
    fn entry_without_spent() {
        let entry = Entry {
            line_count: 6,
            date: NaiveDate::from_ymd_opt(2020, 8, 10).unwrap(),
            effective_date: NaiveDate::from_ymd_opt(2020, 8, 11).unwrap(),
            payee: "Super gas".to_string(),
            amount: dec!(52.10),
            category: "Service stations".to_string(),
            spent: None,
            exchange: None,
            fee: None,
        };

        assert_eq!(
            Some("Super gas"),
            entry.as_entity("CHF").str_field(StrField::Payee)
        );
        assert_eq!(
            Some("Service stations"),
            entry.as_entity("CHF").str_field(StrField::Category)
        );
        assert_eq!(
            None,
            entry
                .as_entity("CHF")
                .str_field(StrField::SecondaryCommodity)
        );
    }

    #[test]
    fn entry_with_spent() {
        let entry = Entry {
            line_count: 6,
            date: NaiveDate::from_ymd_opt(2020, 8, 10).unwrap(),
            effective_date: NaiveDate::from_ymd_opt(2020, 8, 11).unwrap(),
            payee: "Super gas".to_string(),
            amount: dec!(52.10),
            category: "Service stations".to_string(),
            spent: Some(OwnedAmount {
                value: dec!(46.88),
                commodity: "EUR".to_string(),
            }),
            exchange: Some(Exchange {
                rate: dec!(1.092432),
                rate_date: NaiveDate::from_ymd_opt(2020, 8, 9).unwrap(),
                equivalent: OwnedAmount {
                    value: dec!(51.20),
                    commodity: "CHF".to_string(),
                },
            }),
            fee: Some(Fee {
                percent: dec!(1.75),
                amount: OwnedAmount {
                    value: dec!(0.90),
                    commodity: "CHF".to_string(),
                },
            }),
        };

        assert_eq!(
            Some("CHF"),
            entry.as_entity("CHF").str_field(StrField::Commodity)
        );
        assert_eq!(
            Some("EUR"),
            entry
                .as_entity("CHF")
                .str_field(StrField::SecondaryCommodity)
        );
    }
}
