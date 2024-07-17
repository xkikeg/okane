//! eval module contains functions for Ledger file evaluation.

mod book_keeping;
mod context;
mod error;
mod eval;
mod intern;

use std::borrow::Borrow;

pub use book_keeping::{process, Balance, Posting, Transaction};
pub use context::{Account, ReportContext};
pub use error::ReportError;

use crate::{load, repl::LedgerEntry};

/// Returns all accounts for the given LedgerEntry.
/// WARNING: interface are subject to change.
pub fn accounts<'ctx, L, F>(
    ctx: &'ctx mut context::ReportContext,
    loader: L,
) -> Result<Vec<Account<'ctx>>, load::LoadError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    loader.borrow().load_repl(|_path, _ctx, entry| {
        if let LedgerEntry::Txn(txn) = entry {
            for posting in &txn.posts {
                ctx.accounts.intern(&posting.account);
            }
        }
        Ok::<(), load::LoadError>(())
    })?;
    Ok(ctx.all_accounts())
}
