use chrono::NaiveDate;

use super::{context::Account, eval::Amount};

/// Evaluated transaction, already processed to have right balance.
// TODO: Rename it to EvaluatedTxn?
#[derive(Debug, PartialEq, Eq)]
pub struct Transaction<'ctx> {
    pub date: NaiveDate,
    // Posting in the transaction.
    // Note this MUST be a Box instead of &[Posting],
    // as Posting is a [Drop] and we can't skip calling Drop,
    // otherwise we leave allocated memory for Amount HashMap.
    pub postings: bumpalo::boxed::Box<'ctx, [Posting<'ctx>]>,
}

/// Evaluated posting of the transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting<'ctx> {
    pub account: Account<'ctx>,
    /// Note this Amount is not PostingAmount,
    /// as deduced posting may have non-single commodity amount.
    pub amount: Amount<'ctx>,
}
