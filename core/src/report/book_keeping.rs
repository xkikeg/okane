//! Contains book keeping logics to process the input stream,
//! and convert them into a processed Transactions.

use std::borrow::Borrow;

use bumpalo::collections as bcc;
use chrono::NaiveDate;

use crate::{load, repl};

use super::{
    balance::Balance,
    context::{Account, ReportContext},
    error::{self, ReportError},
    eval::{Amount, EvalError, Evaluable},
    intern::InternError,
};

/// Error related to transaction understanding.
// TODO: Reconsider the error in details.
#[derive(Debug, thiserror::Error)]
pub enum BookKeepError {
    #[error("failed to evaluate the expression")]
    EvalFailure(#[from] EvalError),
    #[error("posting amount must be resolved as a simple value with commodity or zero")]
    ComplexPostingAmount,
    #[error("transaction cannot have multiple postings without amount: {0} {1}")]
    UndeduciblePostingAmount(usize, usize),
    #[error("transaction cannot have unbalanced postings: {0}")]
    UnbalancedPostings(String),
    #[error("failed to register account: {0}")]
    InvalidAccount(#[source] InternError),
    #[error("failed to register commodity: {0}")]
    InvalidCommodity(#[source] InternError),
}

/// Takes the loader, and gives back the all read transactions.
/// Also returns the computed balance, as a side-artifact.
/// Usually this needs to be reordered, so just returning a `Vec`.
pub fn process<'ctx, L, F>(
    ctx: &mut ReportContext<'ctx>,
    loader: L,
) -> Result<(Vec<Transaction<'ctx>>, Balance<'ctx>), ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    let mut accum = ProcessAccumulator::new();
    loader.borrow().load_repl(|path, pctx, entry| {
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
    Ok((accum.txns, accum.balance))
}

struct ProcessAccumulator<'ctx> {
    balance: Balance<'ctx>,
    txns: Vec<Transaction<'ctx>>,
}

impl<'ctx> ProcessAccumulator<'ctx> {
    fn new() -> Self {
        let balance = Balance::default();
        let txns: Vec<Transaction<'ctx>> = Vec::new();
        Self { balance, txns }
    }

    fn process(
        &mut self,
        ctx: &mut ReportContext<'ctx>,
        entry: &repl::LedgerEntry,
    ) -> Result<(), BookKeepError> {
        match entry {
            repl::LedgerEntry::Txn(txn) => {
                self.txns
                    .push(add_transaction(ctx, &mut self.balance, txn)?);
                Ok(())
            }
            repl::LedgerEntry::Account(account) => {
                let canonical = ctx
                    .accounts
                    .insert_canonical(&account.name)
                    .map_err(BookKeepError::InvalidAccount)?;
                for ad in &account.details {
                    if let repl::AccountDetail::Alias(alias) = ad {
                        ctx.accounts
                            .insert_alias(alias, canonical)
                            .map_err(BookKeepError::InvalidAccount)?;
                    }
                }
                Ok(())
            }
            repl::LedgerEntry::Commodity(commodity) => {
                let canonical = ctx
                    .commodities
                    .insert_canonical(&commodity.name)
                    .map_err(BookKeepError::InvalidCommodity)?;
                for cd in &commodity.details {
                    if let repl::CommodityDetail::Alias(alias) = cd {
                        ctx.commodities
                            .insert_alias(alias, canonical)
                            .map_err(BookKeepError::InvalidCommodity)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Evaluated transaction, already processed to have right balance.
// TODO: Rename it to EvaluatedTxn?
#[derive(Debug)]
pub struct Transaction<'ctx> {
    pub date: NaiveDate,
    // Posting in the transaction.
    // Note this MUST be a Box instead of &[Posting],
    // as Posting is a [Drop] and we can't skip calling Drop,
    // otherwise we leave allocated memory for Amount HashMap.
    pub postings: bumpalo::boxed::Box<'ctx, [Posting<'ctx>]>,
}

/// Evaluated posting of the transaction.
#[derive(Debug, Clone)]
pub struct Posting<'ctx> {
    pub account: Account<'ctx>,
    pub amount: Amount<'ctx>,
}

/// Adds a repl transaction, and converts it into a processed Transaction.
fn add_transaction<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    bal: &mut Balance<'ctx>,
    txn: &repl::Transaction,
) -> Result<Transaction<'ctx>, BookKeepError> {
    let mut postings = bcc::Vec::with_capacity_in(txn.posts.len(), ctx.arena);
    let mut unfilled: Option<usize> = None;
    let mut balance = Amount::default();
    for (i, posting) in txn.posts.iter().enumerate() {
        let account = ctx.accounts.ensure(&posting.account);
        let posting = match (&posting.amount, &posting.balance) {
            (None, None) => {
                if let Some(j) = unfilled.replace(i) {
                    Err(BookKeepError::UndeduciblePostingAmount(j, i))
                } else {
                    Ok(Posting {
                        account,
                        amount: Amount::default(),
                    })
                }
            }
            (None, Some(balance_constraints)) => {
                let current: Amount = balance_constraints.eval(ctx)?.try_into()?;
                let prev = bal.set_balance(account, current.clone());

                Ok(Posting {
                    account,
                    amount: current - prev,
                })
            }
            (Some(amount), _) => {
                // TODO: add balance constraints check.
                let amount: Amount = amount.amount.eval(ctx)?.try_into()?;
                bal.increment(account, amount.clone());
                balance += amount.clone();
                Ok(Posting { account, amount })
            }
        }?;
        postings.push(posting);
    }
    if let Some(u) = unfilled {
        let deduced = balance.clone().negate();
        postings[u].amount = deduced.clone();
        bal.increment(postings[u].account, deduced);
    } else if !balance.is_zero() {
        // TODO: restore balance checks here.
        // let's ignore this for now, as we're not checking balance properly.
        // This should account for lot price and cost.
        // return Err(BookKeepError::UnbalancedPostings(format!(
        //     "{}",
        //     balance.as_inline_display()
        // )));
    }
    Ok(Transaction {
        date: txn.date,
        postings: postings.into_boxed_slice(),
    })
}
