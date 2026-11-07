use std::borrow::Borrow;
use std::path::PathBuf;

use crate::{load, syntax};

use super::balance::Balance;
use super::book_keeping::{self, BookKeepError};
use super::context::ReportContext;
use super::error::{self, ReportError};
use super::price_db::PriceRepositoryBuilder;
use super::query::Ledger;
use super::tag::TagEntry;
use super::transaction::Transaction;

/// Options to control process behavior.
#[derive(Debug, Default)]
// TODO: non_exhaustive
pub struct ProcessOptions {
    /// Path to the price DB file.
    pub price_db_path: Option<PathBuf>,
}

/// Takes the loader, and gives back the all read transactions.
/// Also returns the computed balance, as a side-artifact.
/// Usually this needs to be reordered, so just returning a `Vec`.
pub fn process<'ctx, L, F>(
    ctx: &mut ReportContext<'ctx>,
    loader: L,
    options: &ProcessOptions,
) -> Result<Ledger<'ctx>, ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    let mut accum = ProcessAccumulator::new();
    loader.borrow().load(|path, pctx, entry| {
        accum.process(ctx, entry).map_err(|berr| {
            ReportError::BookKeep(
                berr,
                error::ErrorContext::new(
                    loader.borrow().error_style().clone(),
                    path.to_owned(),
                    pctx,
                ),
            )
        })
    })?;
    ctx.account_tree.construct(&ctx.accounts);
    if let Some(price_db_path) = options.price_db_path.as_deref() {
        accum
            .price_repos
            .load_price_db(ctx, loader.borrow().filesystem(), price_db_path)?;
    }
    Ok(Ledger {
        arena: ctx.arena,
        transactions: accum.txns,
        date_sorted_txns: None,
        raw_balance: accum.balance,
        price_repos: accum.price_repos.build(),
    })
}

struct ProcessAccumulator<'ctx> {
    balance: Balance<'ctx>,
    txns: Vec<Transaction<'ctx>>,
    price_repos: PriceRepositoryBuilder<'ctx>,
    /// Stack of tags from the `apply tag` directives in effect.
    applied_tags: Vec<TagEntry<'ctx>>,
}

impl<'ctx> ProcessAccumulator<'ctx> {
    fn new() -> Self {
        Self {
            balance: Balance::default(),
            txns: Vec::new(),
            price_repos: PriceRepositoryBuilder::default(),
            applied_tags: Vec::new(),
        }
    }

    fn process(
        &mut self,
        ctx: &mut ReportContext<'ctx>,
        entry: &syntax::tracked::LedgerEntry,
    ) -> Result<(), BookKeepError> {
        match &entry.statement {
            syntax::LedgerStatement::Txn(txn) => {
                self.txns.push(book_keeping::add_transaction(
                    ctx,
                    &mut self.price_repos,
                    &mut self.balance,
                    &self.applied_tags,
                    txn,
                )?);
                Ok(())
            }
            syntax::LedgerStatement::ApplyTag(apply) => {
                self.applied_tags.push(book_keeping::tag_entry(
                    ctx,
                    &apply.key,
                    apply.value.as_ref(),
                ));
                Ok(())
            }
            syntax::LedgerStatement::EndApplyTag => self
                .applied_tags
                .pop()
                .map(|_| ())
                .ok_or(BookKeepError::UnmatchedEndApplyTag),
            syntax::LedgerStatement::Account(account) => process_account(ctx, account),
            syntax::LedgerStatement::Commodity(commodity) => process_commodity(ctx, commodity),
            _ => Ok(()),
        }
    }
}

pub fn process_account<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    account: &syntax::AccountDeclaration<'_>,
) -> Result<(), BookKeepError> {
    let canonical = ctx.accounts.ensure(&account.name);
    for ad in &account.details {
        if let syntax::AccountDetail::Alias(alias) = ad {
            ctx.accounts
                .register_alias(alias, canonical)
                .map_err(|_| BookKeepError::InvalidAccountAlias(alias.to_string()))?;
        }
    }
    Ok(())
}

fn process_commodity<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    commodity: &syntax::CommodityDeclaration<'_>,
) -> Result<(), BookKeepError> {
    let canonical = ctx.commodities.ensure(&commodity.name);
    for cd in &commodity.details {
        match cd {
            syntax::CommodityDetail::Alias(alias) => {
                ctx.commodities
                    .register_alias(alias, canonical)
                    .map_err(|_| BookKeepError::InvalidCommodityAlias(alias.to_string()))?;
            }
            syntax::CommodityDetail::Format(format_amount) => {
                ctx.commodities.set_format(canonical, format_amount.value);
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::path::Path;

    use assert_matches::assert_matches;
    use bumpalo::Bump;

    use std::path::PathBuf;

    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use super::super::account::{AccountAggregate, AccountTreeKey};

    fn process_str<'ctx>(
        ctx: &mut ReportContext<'ctx>,
        input: &str,
    ) -> Result<Ledger<'ctx>, ReportError> {
        let path = PathBuf::from("path/to/file.ledger");
        let fake = hashmap! { path.clone() => input.as_bytes().to_vec() };
        let loader: load::Loader<load::FakeFileSystem> = load::Loader::new(path, fake.into());
        process(ctx, loader, &ProcessOptions::default())
    }

    /// Returns `key` or `key=value` of each tag, sorted by key.
    fn tag_strs(tags: &super::super::TagSet<'_>) -> Vec<String> {
        tags.iter()
            .map(|e| match e.value {
                None => e.key.as_str().to_string(),
                Some(v) => format!("{}={}", e.key.as_str(), v),
            })
            .collect()
    }

    fn is_duplicate_tag(err: &ReportError, want_key: &str, on_posting: bool) -> bool {
        matches!(
            err,
            ReportError::BookKeep(BookKeepError::DuplicateTag { key, posting, .. }, _)
                if key == want_key && posting.is_some() == on_posting
        )
    }

    #[test]
    fn process_constructs_account_tree() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/report/multi_commodity.ledger");

        let _ = process(
            &mut ctx,
            load::new_loader(input),
            &ProcessOptions::default(),
        )
        .unwrap();

        let parent = ctx.account_aggregate("Assets:Banks").unwrap();
        let child1 = ctx.account_aggregate("Assets:Banks:Swiss Bank").unwrap();
        let child2 = ctx.account_aggregate("Assets:Banks:あおによし").unwrap();
        assert_matches!(parent, AccountAggregate::Ancestor(_));
        assert_matches!(child1, AccountAggregate::Account(_));
        assert_matches!(child2, AccountAggregate::Account(_));
        let children: &[_] = &[child1, child2];

        assert_eq!(Some(children), ctx.account_tree.children(parent));
        assert_eq!(
            Some(AccountTreeKey::Descendant(parent)),
            ctx.account_tree.parent(child1)
        );
    }

    #[test]
    fn process_collects_transaction_and_posting_tags() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input = indoc! {"
            2024/01/01 Shop
                ; :food:daily:
                ; trip: Kyoto
                Expenses:Food       10 CHF  ; :snack:
                Assets:Bank         ; Payee: ATM
        "};
        let ledger = process_str(&mut ctx, input).unwrap();
        let txn = ledger.transactions().next().unwrap();
        assert_eq!(vec!["daily", "food", "trip=Kyoto"], tag_strs(&txn.tags));
        assert_eq!(vec!["snack"], tag_strs(&txn.postings[0].tags));
        assert_eq!(vec!["Payee=ATM"], tag_strs(&txn.postings[1].tags));
    }

    #[test]
    fn process_applies_nested_apply_tags() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input = indoc! {"
            apply tag trip: Kyoto

            2024/01/01 Train
                Expenses:Travel     10 CHF
                Assets:Bank

            apply tag business

            2024/01/02 Hotel
                ; :hotel:
                Expenses:Travel     20 CHF
                Assets:Bank

            end apply tag

            2024/01/03 Dinner
                Expenses:Food       30 CHF
                Assets:Bank

            end apply tag

            2024/01/04 Home
                Expenses:Food       40 CHF
                Assets:Bank
        "};
        let ledger = process_str(&mut ctx, input).unwrap();
        let got: Vec<Vec<String>> = ledger.transactions().map(|t| tag_strs(&t.tags)).collect();
        assert_eq!(
            vec![
                vec!["trip=Kyoto".to_string()],
                vec![
                    "business".to_string(),
                    "hotel".to_string(),
                    "trip=Kyoto".to_string()
                ],
                vec!["trip=Kyoto".to_string()],
                vec![],
            ],
            got
        );
        // apply tag goes to the transaction, not to its postings.
        let first = ledger.transactions().next().unwrap();
        assert!(first.postings.iter().all(|p| p.tags.is_empty()));
    }

    #[test]
    fn process_allows_posting_tag_overriding_transaction_tag() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input = indoc! {"
            2024/01/01 Shop
                ; trip: Kyoto
                Expenses:Food       10 CHF  ; trip: Osaka
                Assets:Bank
        "};
        let ledger = process_str(&mut ctx, input).unwrap();
        let txn = ledger.transactions().next().unwrap();
        assert_eq!(vec!["trip=Kyoto"], tag_strs(&txn.tags));
        assert_eq!(vec!["trip=Osaka"], tag_strs(&txn.postings[0].tags));
    }

    #[test]
    fn process_rejects_duplicate_transaction_tag() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input = indoc! {"
            2024/01/01 Shop
                ; :trip:
                ; trip: Kyoto
                Expenses:Food       10 CHF
                Assets:Bank
        "};
        let err = process_str(&mut ctx, input).unwrap_err();
        assert!(is_duplicate_tag(&err, "trip", false), "{err:?}");
    }

    #[test]
    fn process_rejects_duplicate_posting_tag() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input = indoc! {"
            2024/01/01 Shop
                Expenses:Food       10 CHF  ; :snack:snack:
                Assets:Bank
        "};
        let err = process_str(&mut ctx, input).unwrap_err();
        assert!(is_duplicate_tag(&err, "snack", true), "{err:?}");
    }

    #[test]
    fn process_rejects_transaction_tag_duplicating_apply_tag() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let input = indoc! {"
            apply tag trip: Kyoto

            2024/01/01 Shop
                ; trip: Osaka
                Expenses:Food       10 CHF
                Assets:Bank

            end apply tag
        "};
        let err = process_str(&mut ctx, input).unwrap_err();
        assert!(is_duplicate_tag(&err, "trip", false), "{err:?}");
    }

    #[test]
    fn process_rejects_unmatched_end_apply_tag() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let err = process_str(&mut ctx, "end apply tag\n").unwrap_err();
        assert_matches!(
            err,
            ReportError::BookKeep(BookKeepError::UnmatchedEndApplyTag, _)
        );
    }
}
