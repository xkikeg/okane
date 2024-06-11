use super::{
    amounts::{Amount, Balance},
    context::EvalContext,
    error::{BalanceError, EvalError},
    types::Account,
};

use crate::{
    eval::amounts::{Evaluable, PartialAmount},
    repl,
};

use bumpalo::collections as bcc;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Processed transaction
#[derive(Debug, Clone)]
pub struct Transaction<'arena> {
    pub date: NaiveDate,
    pub postings: &'arena [Posting<'arena>],
}

/// Processed posting.
#[derive(Debug, Clone)]
pub struct Posting<'arena> {
    pub account: Account<'arena>,
    pub amount: Amount<'arena>,
}

pub fn balanced_txn<'arena>(
    ctx: &mut EvalContext<'arena>,
    total_balance: &mut Balance<'arena>,
    txn: &repl::Transaction,
) -> Result<Transaction<'arena>, BalanceError> {
    let mut postings = bcc::Vec::with_capacity_in(txn.posts.len(), ctx.allocator);
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
                let updated_balance = amount_from_partial(balance_constraints.eval(ctx)?)?;
                let prev_balance = total_balance
                    .accounts
                    .insert(account, updated_balance)
                    .unwrap_or(Amount::default());
                Ok(Posting {
                    account,
                    amount: prev_balance.negate(),
                })
            }
            (Some(amount), _) => {
                // TODO: add balance constraints check.
                let amount = amount_from_partial(amount.amount.eval(ctx)?)?;
                balance += amount.clone();
                Ok(Posting { account, amount })
            }
        }?;
        postings.push(posting);
    }
    if let Some(u) = unfilled {
        let deduced = balance.clone().negate();
        postings[u].amount = deduced;
    } else if !balance.is_zero() {
        // TODO: revert checks
        // let's ignore this for now, as we're not checking balance properly.
        // This should account for lot price and cost.
        // return Err(BalanceError::UnbalancedPostings(format!("{}", balance.as_inline())));
    }
    Ok(Transaction {
        date: txn.date.clone(),
        postings: postings.into_bump_slice(),
    })
}

fn amount_from_partial(x: PartialAmount) -> Result<Amount, BalanceError> {
    match x {
        PartialAmount::Number(x) if x.is_zero() => Ok(Amount::default()),
        PartialAmount::Number(_) => Err(BalanceError::ComplexPostingAmount),
        PartialAmount::Commodities(amount) => Ok(amount),
    }
}
