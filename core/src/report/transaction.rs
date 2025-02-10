use chrono::NaiveDate;

use super::{
    context::Account,
    eval::{Amount, SingleAmount},
};

/// Evaluated transaction, already processed to have right balance.
// TODO: Rename it to EvaluatedTxn?
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct Posting<'ctx> {
    /// Account of the posting.
    pub account: Account<'ctx>,

    /// Amount of the posting described in the Ledger.
    // Note this is not PostingAmount,
    // as deduced posting may have non-single commodity amount.
    pub amount: Amount<'ctx>,

    /// Amount of the posting in cost basis.
    /// Some time this is useful for a few use cases:
    /// - To balance within the transaction, we prefer this amount.
    pub converted_amount: Option<SingleAmount<'ctx>>,
}
