use crate::import::extract::{self, StrField};

/// Record is a wrapper to represent a CSV line.
#[derive(Debug, Copy, Clone)]
pub struct Record<'a> {
    pub payee: &'a str,
    pub category: Option<&'a str>,
    pub commodity: &'a str,
    pub secondary_commodity: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct CsvFormat;

impl extract::EntityFormat for CsvFormat {
    fn name(&self) -> &'static str {
        "CSV"
    }

    fn has_camt_transaction_code(&self) -> bool {
        false
    }

    fn has_str_field(&self, field: StrField) -> bool {
        match field {
            StrField::Payee => true,
            StrField::Category => true,
            StrField::Commodity => true,
            StrField::SecondaryCommodity => true,
            StrField::Camt(_) => false,
        }
    }
}

impl<'a> extract::Entity<'a> for &'a Record<'a> {
    fn str_field(&self, field: StrField) -> Option<&'a str> {
        match field {
            StrField::Payee => Some(self.payee),
            StrField::Category => self.category,
            StrField::Commodity => Some(self.commodity),
            StrField::SecondaryCommodity => self.secondary_commodity,
            StrField::Camt(_) => None,
        }
    }
}
