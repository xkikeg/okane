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
pub enum BookKeepError {
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
pub fn process<'ctx, L, F>(
    ctx: &mut ReportContext<'ctx>,
    loader: L,
) -> Result<(Vec<Transaction<'ctx>>, Balance<'ctx>), ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    let mut balance = Balance::default();
    let mut txns: Vec<Transaction<'ctx>> = Vec::new();
    loader.borrow().load_repl(|_path, _ctx, entry| {
        match entry {
            repl::LedgerEntry::Txn(txn) => txns.push(add_transaction(ctx, &mut balance, txn)?),
            repl::LedgerEntry::Account(account) => {
                for ad in &account.details {
                    if let repl::AccountDetail::Alias(alias) = ad {
                        log::warn!(
                            "account {} has alias {}, which is not supported yet",
                            account.name,
                            alias
                        );
                    }
                }
            }
            repl::LedgerEntry::Commodity(commodity) => {
                for cd in &commodity.details {
                    if let repl::CommodityDetail::Alias(alias) = cd {
                        log::warn!(
                            "commodity {} has alias {}, which is not supported yet",
                            commodity.name,
                            alias
                        );
                    }
                }
            }
            _ => {}
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
    accounts: HashMap<Account<'ctx>, Amount<'ctx>>,
}

impl<'ctx> Balance<'ctx> {
    /// Adds a particular account value, and returns the updated balance.
    pub fn increment(&mut self, account: Account<'ctx>, amount: Amount<'ctx>) -> Amount<'ctx> {
        let curr: &mut Amount = self.accounts.entry(account).or_default();
        *curr += amount;
        curr.clone()
    }

    /// Sets the particular account's balance, and returns the previous balance.
    pub fn set_balance(&mut self, account: Account<'ctx>, amount: Amount<'ctx>) -> Amount<'ctx> {
        self.accounts.insert(account, amount).unwrap_or_default()
    }

    /// Gets the balance of the given account.
    pub fn get_balance(&self, account: &Account<'ctx>) -> Option<&Amount<'ctx>> {
        self.accounts.get(account)
    }
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
        let account = ctx.accounts.intern(&posting.account);
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
        // return Err(BalanceError::UnbalancedPostings(format!("{}", balance.as_inline())));
    }
    Ok(Transaction {
        date: txn.date,
        postings: postings.into_bump_slice(),
    })
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn balance_gives_zero_amount_when_not_initalized() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let balance = Balance::default();
        assert_eq!(balance.get_balance(&ctx.accounts.intern("Expenses")), None);
    }

    #[test]
    fn test_balance_increment_adds_value() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let mut balance = Balance::default();
        let updated = balance.increment(
            ctx.accounts.intern("Expenses"),
            Amount::from_value(dec!(1000), ctx.commodities.intern("JPY")),
        );

        assert_eq!(
            updated,
            Amount::from_value(dec!(1000), ctx.commodities.intern("JPY"))
        );
        assert_eq!(
            balance.get_balance(&ctx.accounts.intern("Expenses")),
            Some(&updated)
        );

        let updated = balance.increment(
            ctx.accounts.intern("Expenses"),
            Amount::from_value(dec!(-1000), ctx.commodities.intern("JPY")),
        );

        assert_eq!(updated, Amount::zero());
        assert_eq!(
            balance.get_balance(&ctx.accounts.intern("Expenses")),
            Some(&updated)
        );
    }

    #[test]
    fn test_balance_set_balance() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let mut balance = Balance::default();
        let prev = balance.set_balance(
            ctx.accounts.intern("Expenses"),
            Amount::from_value(dec!(1000), ctx.commodities.intern("JPY")),
        );

        assert_eq!(prev, Amount::zero());
        assert_eq!(
            balance.get_balance(&ctx.accounts.intern("Expenses")),
            Some(&Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
        );

        let prev = balance.set_balance(
            ctx.accounts.intern("Expenses"),
            Amount::from_value(dec!(-1000), ctx.commodities.intern("JPY")),
        );

        assert_eq!(
            prev,
            Amount::from_value(dec!(1000), ctx.commodities.intern("JPY"))
        );
        assert_eq!(
            balance.get_balance(&ctx.accounts.intern("Expenses")),
            Some(&Amount::from_value(
                dec!(-1000),
                ctx.commodities.intern("JPY")
            ))
        );
    }
}
