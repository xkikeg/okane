use std::collections::{HashMap, HashSet};

use super::{Entity, EntityFormat, StrField, xmlnode};

/// [`EntityFormat`] for unit tests.
#[derive(Debug, Default)]
pub struct TestFormat {
    pub has_all: bool,
    pub has_camt: bool,
    pub has_str: HashSet<StrField>,
}

impl TestFormat {
    /// Returns [`TestFormat`] which doesn't support any fields.
    pub fn none() -> Self {
        TestFormat::default()
    }

    /// Returns [`TestFormat`] instance accepting any fields.
    pub fn all() -> Self {
        Self {
            has_all: true,
            has_camt: false,
            has_str: HashSet::new(),
        }
    }
}

impl EntityFormat for &TestFormat {
    fn name(&self) -> &'static str {
        "test-format"
    }

    fn has_camt_transaction_code(&self) -> bool {
        self.has_all || self.has_camt
    }

    fn has_str_field(&self, field: StrField) -> bool {
        self.has_all || self.has_str.contains(&field)
    }
}

/// [`Entity`] for unit tests.
#[derive(Debug, Default)]
pub(super) struct TestEntity {
    pub camt_txn_code: Option<xmlnode::BankTransactionCode>,
    pub str_fields: HashMap<StrField, String>,
}

impl<'a> Entity<'a> for &'a TestEntity {
    fn camt_transaction_code(&self) -> Option<&'a xmlnode::BankTransactionCode> {
        self.camt_txn_code.as_ref()
    }

    fn str_field(&self, field: StrField) -> Option<&'a str> {
        self.str_fields.get(&field).map(String::as_str)
    }
}
