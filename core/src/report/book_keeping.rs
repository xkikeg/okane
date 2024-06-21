//! Contains book keeping logics to process the input stream,
//! and convert them into a processed Transactions.

use std::{borrow::Borrow, collections::HashMap};

use bumpalo::collections as bcc;
use chrono::NaiveDate;

use crate::{load, repl};

use super::{
    context::ReportContext,
    eval::{Amount, EvalError, Evaluable},
    intern::Account,
    ReportError,
};

/// Error related to transaction understanding.
// TODO: Reconsider the error in details.
#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    #[error("failed to evaluate the expression")]
    EvalFailure(#[from] EvalError),
    #[error("posting amount must be resolved as a simple value with commodity or zero")]
    ComplexPostingAmount,
    #[error("transaction cannot have multiple postings without amount: {0} {1}")]
    UndeduciblePostingAmount(usize, usize),
    #[error("transaction cannot have unbalanced postings: {0}")]
    UnbalancedPostings(String),
}

/// Takes the loader, and gives back the all read transactions.
/// Also returns the computed balance, as a side-artifact.
/// Usually this needs to be reordered, so just returning a `Vec`.
pub fn process<'ctx, L>(
    ctx: &mut ReportContext<'ctx>,
    loader: L,
) -> Result<(Vec<Transaction<'ctx>>, Balance<'ctx>), ReportError>
where
    L: Borrow<load::Loader>,
{
    let mut balance = Balance::default();
    let mut txns: Vec<Transaction<'ctx>> = Vec::new();
    loader.borrow().load_repl(|_path, entry| {
        if let repl::LedgerEntry::Txn(txn) = entry {
            txns.push(balance.add_transaction(ctx, txn)?);
        }
        // TODO: Report file path and location, maybe use ReportError if needed
        Ok::<(), ReportError>(())
    })?;
    Ok((txns, balance))
}

/// Evaluated transaction, already processed to have right balance.
// TODO: Rename it to EvaluatedTxn?
#[derive(Debug, Clone)]
pub struct Transaction<'ctx> {
    pub date: NaiveDate,
    pub postings: &'ctx [Posting<'ctx>],
}

/// Evaluated posting of the transaction.
#[derive(Debug, Clone)]
pub struct Posting<'ctx> {
    pub account: Account<'ctx>,
    pub amount: Amount<'ctx>,
}

/// Balance of all accounts after the accumulated transactions.
#[derive(Debug, Default)]
pub struct Balance<'ctx> {
    // TODO: Make it private.
    pub accounts: HashMap<Account<'ctx>, Amount<'ctx>>,
}

impl<'ctx> Balance<'ctx> {
    /// Adds a particular account value, and returns the previous balance.
    pub fn fetch_add(&mut self, account: Account<'ctx>, amount: Amount<'ctx>) -> Amount<'ctx> {
        let curr = self.accounts.entry(account).or_default();
        let prev = curr.clone();
        *curr += amount;
        prev
    }

    /// Sets the particular account's balance, and returns the previous balance.
    pub fn fetch_set(&mut self, account: Account<'ctx>, amount: Amount<'ctx>) -> Amount<'ctx> {
        self.accounts.insert(account, amount).unwrap_or_default()
    }

    /// Adds a repl transaction, and converts it into a processed Transaction.
    pub fn add_transaction(
        &mut self,
        ctx: &mut ReportContext<'ctx>,
        txn: &repl::Transaction,
    ) -> Result<Transaction<'ctx>, BalanceError> {
        let mut postings = bcc::Vec::with_capacity_in(txn.posts.len(), ctx.arena);
        let mut unfilled: Option<usize> = None;
        let mut balance = Amount::default();
        for (i, posting) in txn.posts.iter().enumerate() {
            let account = ctx.accounts.intern(&posting.account);
            let posting = match (&posting.amount, &posting.balance) {
                (None, None) => {
                    if let Some(j) = unfilled.replace(i) {
                        Err(BalanceError::UndeduciblePostingAmount(j, i))
                    } else {
                        Ok(Posting {
                            account,
                            amount: Amount::default(),
                        })
                    }
                }
                (None, Some(balance_constraints)) => {
                    let mut updated_balance: Amount = balance_constraints.eval(ctx)?.try_into()?;
                    let prev_balance = self.fetch_set(account, updated_balance.clone());
                    // TODO: use - instead of -=
                    updated_balance += prev_balance.negate();
                    Ok(Posting {
                        account,
                        amount: updated_balance,
                    })
                }
                (Some(amount), _) => {
                    // TODO: add balance constraints check.
                    let amount: Amount = amount.amount.eval(ctx)?.try_into()?;
                    self.fetch_add(account, amount.clone());
                    balance += amount.clone();
                    Ok(Posting { account, amount })
                }
            }?;
            postings.push(posting);
        }
        if let Some(u) = unfilled {
            let deduced = balance.clone().negate();
            postings[u].amount = deduced.clone();
            self.fetch_add(postings[u].account, deduced);
        } else if !balance.is_zero() {
            // TODO: restore balance checks here.
            // let's ignore this for now, as we're not checking balance properly.
            // This should account for lot price and cost.
            // return Err(BalanceError::UnbalancedPostings(format!("{}", balance.as_inline())));
        }
        Ok(Transaction {
            date: txn.date.clone(),
            postings: postings.into_bump_slice(),
        })
    }
}
