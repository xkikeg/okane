//! eval module contains functions for Ledger file evaluation.

mod book_keeping;
mod context;
mod error;
mod eval;
mod intern;

use std::borrow::Borrow;

pub use book_keeping::{process, Balance, Posting, Transaction};
pub use context::ReportContext;
pub use error::ReportError;
pub use intern::Account;

use crate::{load, repl::LedgerEntry};

/// Returns all accounts for the given LedgerEntry.
/// WARNING: interface are subject to change.
pub fn accounts<'ctx, L>(
    ctx: &'ctx mut context::ReportContext,
    loader: L,
) -> Result<Vec<intern::Account<'ctx>>, load::LoadError>
where
    L: Borrow<load::Loader>,
{
    loader.borrow().load_repl(|_path, entry| {
        if let LedgerEntry::Txn(txn) = entry {
            for posting in &txn.posts {
                ctx.accounts.intern(&posting.account);
            }
        }
        Ok::<(), load::LoadError>(())
    })?;
    Ok(ctx.all_accounts())
}
