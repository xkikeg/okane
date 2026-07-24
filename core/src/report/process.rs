use std::borrow::Borrow;
use std::path::PathBuf;

use crate::{load, syntax};

use super::balance::Balance;
use super::book_keeping::{self, BookKeepError};
use super::context::ReportContext;
use super::error::{self, ReportError};
use super::price_db::PriceRepositoryBuilder;
use super::query::Ledger;
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
}

impl<'ctx> ProcessAccumulator<'ctx> {
    fn new() -> Self {
        Self {
            balance: Balance::default(),
            txns: Vec::new(),
            price_repos: PriceRepositoryBuilder::default(),
        }
    }

    fn process(
        &mut self,
        ctx: &mut ReportContext<'ctx>,
        entry: &syntax::tracked::LedgerEntry,
    ) -> Result<(), BookKeepError> {
        match entry {
            syntax::LedgerEntry::Txn(txn) => {
                self.txns.push(book_keeping::add_transaction(
                    ctx,
                    &mut self.price_repos,
                    &mut self.balance,
                    txn,
                )?);
                Ok(())
            }
            syntax::LedgerEntry::Account(account) => process_account(ctx, account),
            syntax::LedgerEntry::Commodity(commodity) => process_commodity(ctx, commodity),
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
