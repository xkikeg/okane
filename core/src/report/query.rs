//! Provides query of transactions / balances on the processed [Ledger] instance.

use std::{borrow::Cow, collections::HashSet};

use super::{
    balance::Balance,
    context::{Account, ReportContext},
    transaction::{Posting, Transaction},
};

/// Contains processed transactions, so that users can query information.
pub struct Ledger<'ctx> {
    pub(super) transactions: Vec<Transaction<'ctx>>,
    pub(super) raw_balance: Balance<'ctx>,
}

/// Query to list postings matching the criteria.
#[derive(Debug)]
pub struct PostingQuery {
    /// Select the specified account if specified.
    /// Note this will be changed to list of regex eventually.
    pub account: Option<String>,
}

impl<'ctx> Ledger<'ctx> {
    /// Returns iterator for all transactions.
    pub fn transactions(&self) -> impl Iterator<Item = &Transaction<'ctx>> {
        self.transactions.iter()
    }

    /// Returns all postings following the queries.
    pub fn postings<'a>(
        &'a self,
        ctx: &ReportContext<'ctx>,
        query: &PostingQuery,
    ) -> Vec<&'a Posting<'ctx>> {
        // compile them into compiled query.
        let af = AccountFilter::new(ctx, query.account.as_deref());
        let af = match af {
            None => return Vec::new(),
            Some(af) => af,
        };
        self.transactions()
            .flat_map(|txn| &*txn.postings)
            .filter(|x| af.is_match(&x.account))
            .collect()
    }

    /// Returns a balance matching the given query.
    /// Note that currently we don't have the query,
    /// that will be implemented soon.
    pub fn balance(&self) -> Cow<'_, Balance<'ctx>> {
        Cow::Borrowed(&self.raw_balance)
    }
}

enum AccountFilter<'ctx> {
    Any,
    Set(HashSet<Account<'ctx>>),
}

impl<'ctx> AccountFilter<'ctx> {
    /// Creates a new instance, unless there's no matching account.
    fn new(ctx: &ReportContext<'ctx>, filter: Option<&str>) -> Option<Self> {
        let filter = match filter {
            None => return Some(AccountFilter::Any),
            Some(filter) => filter,
        };
        let targets: HashSet<_> = ctx
            .all_accounts_unsorted()
            .filter(|x| x.as_str() == filter)
            .collect();
        if targets.is_empty() {
            return None;
        }
        Some(AccountFilter::Set(targets))
    }

    fn is_match(&self, account: &Account<'ctx>) -> bool {
        match self {
            AccountFilter::Any => true,
            AccountFilter::Set(targets) => targets.contains(account),
        }
    }
}
