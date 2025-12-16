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

impl<'a> extract::Entity<'a> for &'a Entry {
    fn str_field(&self, field: StrField) -> Option<&'a str> {
        match field {
            StrField::Camt(_) => None,
            StrField::Payee => Some(&self.payee),
            StrField::Category => Some(&self.category),
            StrField::SecondaryCommodity => self.spent.as_ref().map(|x| x.commodity.as_str()),
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

        assert_eq!(Some("Super gas"), (&entry).str_field(StrField::Payee));
        assert_eq!(
            Some("Service stations"),
            (&entry).str_field(StrField::Category)
        );
        assert_eq!(None, (&entry).str_field(StrField::SecondaryCommodity));
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
            Some("EUR"),
            (&entry).str_field(StrField::SecondaryCommodity)
        );
    }
}
