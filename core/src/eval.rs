//! eval module contains functions for Ledger file evaluation.

mod amounts;
pub mod context;
pub mod types;

use std::path::PathBuf;

pub use amounts::EvalError;
use amounts::Evaluable;

use crate::repl::{
    self,
    parser::{parse_ledger, ParseLedgerError},
    LedgerEntry,
};

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

/// Returns total amount per accounts.
/// Note this function will be removed by the next release.
pub fn total_balance<'a, 'ctx, I>(
    ctx: &'ctx mut context::EvalContext,
    entries: I,
) -> Result<amounts::Balance<'ctx>, amounts::EvalError>
where
    I: IntoIterator<Item = &'a repl::LedgerEntry>,
{
    let mut balance = amounts::Balance::default();
    for entry in entries {
        if let LedgerEntry::Txn(txn) = entry {
            for posting in &txn.posts {
                let account = ctx.accounts.intern(&posting.account);
                if let Some(x) = &posting.amount {
                    let posting_amount = x.amount.eval(ctx)?;
                    match posting_amount {
                        amounts::PartialAmount::Number(x) => {
                            if !x.is_zero() {
                                // TODO: DO NOT USE default context.
                                log::error!(
                                    " amount without commodity {}: {}",
                                    x,
                                    repl::display::DisplayContext::default().as_display(txn)
                                );
                                return Err(amounts::EvalError::ComplexPostingAmount);
                            }
                        }
                        amounts::PartialAmount::Commodities(x) => {
                            *balance.accounts.entry(account).or_default() += x;
                        }
                    }
                }
            }
        }
    }
    Ok(balance)
}
