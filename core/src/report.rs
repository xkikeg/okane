//! eval module contains functions for Ledger file evaluation.

mod balance;
mod book_keeping;
mod commodity;
mod context;
mod error;
mod eval;
mod price_db;
mod process;
pub mod query;
mod transaction;

use std::borrow::Borrow;

pub use balance::Balance;
pub use commodity::{Commodity, CommodityStore, CommodityTag, OwnedCommodity};
pub use context::{Account, ReportContext};
pub use error::ReportError;
pub use eval::{Amount, SingleAmount};
pub use price_db::LoadError;
pub use process::{process, ProcessOptions};
pub use transaction::{Posting, Transaction};

use crate::{load, syntax::plain::LedgerEntry};

/// Returns all accounts for the given LedgerEntry.
/// WARNING: interface are subject to change.
pub fn accounts<'ctx, L, F>(
    ctx: &'_ mut context::ReportContext<'ctx>,
    loader: L,
) -> Result<Vec<Account<'ctx>>, ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    loader.borrow().load(|path, pctx, entry| {
        match entry {
            LedgerEntry::Account(account) => {
                process::process_account(ctx, account).map_err(|berr| {
                    ReportError::BookKeep(
                        berr,
                        error::ErrorContext::new(
                            loader.borrow().error_style().clone(),
                            path.to_owned(),
                            pctx,
                        ),
                    )
                })?
            }
            LedgerEntry::Txn(txn) => {
                for posting in &txn.posts {
                    ctx.accounts.ensure(&posting.account);
                }
            }
            _ => (),
        }
        Ok::<(), ReportError>(())
    })?;
    Ok(ctx.all_accounts())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bumpalo::Bump;
    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::{load, report};

    #[test]
    fn accounts_gives_all_accounts() {
        let fake = hashmap! {
            PathBuf::from("path/to/root.ledger") => indoc! {"
                account Expenses:Food
                   alias Expenses:Bar

                account Expenses:Bar
                   alias Expenses:Baz

                2026/01/01 drink
                   Expenses:Baz     1,000 JPY
                   Assets:Bank:X
            "}.as_bytes().to_vec(),
        };
        let loader = load::Loader::new(
            PathBuf::from("path/to/root.ledger"),
            load::FakeFileSystem::from(fake),
        );
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);

        let got = report::accounts(&mut ctx, &loader).unwrap();

        assert_eq!(
            vec![
                ctx.account("Assets:Bank:X").unwrap(),
                ctx.account("Expenses:Food").unwrap()
            ],
            got,
        );
    }
}
