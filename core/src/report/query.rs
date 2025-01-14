//! Provides query of transactions / balances on the processed [Ledger] instance.

use super::{balance::Balance, transaction::Transaction};

/// Contains processed transactions, so that users can query information.
pub struct Ledger<'ctx> {
    pub(super) transactions: Vec<Transaction<'ctx>>,
    pub(super) raw_balance: Balance<'ctx>,
}

/// Query to list transactions matching the criteria.
pub struct TransactionQuery {}

impl<'ctx> Ledger<'ctx> {
    /// Returns iterator for all transactions.
    pub fn transactions(&self) -> impl Iterator<Item = &Transaction<'ctx>> {
        self.transactions.iter()
    }

    /// Returns a reference for the raw balance (commodity unconverted balance).
    /// Note this API will be removed soon.
    pub fn balance(&self) -> &Balance<'ctx> {
        &self.raw_balance
    }
}
