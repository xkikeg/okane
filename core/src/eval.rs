//! eval module contains functions for Ledger file evaluation.

pub mod context;
pub mod types;

use std::path::Path;

use crate::{load, repl::LedgerEntry};

/// Returns all accounts for the given LedgerEntry.
/// Note this function will be removed by the next release.
pub fn accounts<'ctx>(
    ctx: &'ctx mut context::EvalContext,
    path: &Path,
) -> Result<Vec<types::Account<'ctx>>, load::LoadError> {
    load::load_repl(path, |_path, entry| {
        if let LedgerEntry::Txn(txn) = entry {
            for posting in &txn.posts {
                ctx.accounts.intern(&posting.account);
            }
        }
        Ok::<(), load::LoadError>(())
    })?;
    Ok(ctx.all_accounts())
}
