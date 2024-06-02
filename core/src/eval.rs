//! eval module contains functions for Ledger file evaluation.

mod amounts;
pub mod context;
pub mod types;

use crate::repl::{self, LedgerEntry};

/// Returns all accounts for the given LedgerEntry.
/// Note this function will be removed by the next release.
pub fn accounts<'ctx>(
    ctx: &'ctx mut context::EvalContext,
    entries: &[repl::LedgerEntry],
) -> Vec<types::Account<'ctx>> {
    for entry in entries {
        if let LedgerEntry::Txn(txn) = entry {
            for posting in &txn.posts {
                ctx.accounts.intern(&posting.account);
            }
        }
    }
    ctx.all_accounts()
}
