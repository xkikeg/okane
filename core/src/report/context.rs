use bumpalo::Bump;

use crate::report::AccountAggregate;

use super::account::{Account, AccountStore, AccountTree};
use super::commodity::{CommodityStore, CommodityTag};

/// Context object extensively used across Ledger file evaluation.
pub struct ReportContext<'ctx> {
    pub(super) arena: &'ctx Bump,
    pub(super) accounts: AccountStore<'ctx>,
    pub(super) account_tree: AccountTree<'ctx>,
    pub(super) commodities: CommodityStore<'ctx>,
}

impl<'ctx> ReportContext<'ctx> {
    /// Create a new instance of `ReportContext`.
    pub fn new(arena: &'ctx Bump) -> Self {
        let accounts = AccountStore::new(arena);
        let account_tree = AccountTree::new(arena);
        let commodities = CommodityStore::new(arena);
        Self {
            arena,
            accounts,
            account_tree,
            commodities,
        }
    }

    /// Returns all accounts, sorted as string order.
    pub(super) fn all_accounts_unsorted(&self) -> impl Iterator<Item = Account<'ctx>> + '_ {
        self.accounts.iter()
    }
    /// Returns all accounts, sorted as string order.
    pub(super) fn all_accounts(&self) -> Vec<Account<'ctx>> {
        let mut r: Vec<Account<'ctx>> = self.all_accounts_unsorted().collect();
        r.sort_unstable_by_key(|x| x.as_str());
        r
    }

    /// Returns the given account, or `None` if not found.
    #[inline]
    pub fn account(&self, value: &str) -> Option<Account<'ctx>> {
        self.accounts.resolve(value)
    }

    /// Returns the account aggregate, or `None` if not found.
    #[inline]
    pub fn account_aggregate(&self, value: &str) -> Option<AccountAggregate<'ctx>> {
        match self.account(value) {
            Some(v) => Some(AccountAggregate::Account(v)),
            None => self
                .account_tree
                .resolve_ancestor(value)
                .map(AccountAggregate::Ancestor),
        }
    }

    /// Returns the given commmodity, or `None` if not found.
    #[inline]
    pub fn commodity(&self, value: &str) -> Option<CommodityTag<'ctx>> {
        self.commodities.resolve(value)
    }

    /// Returns [`CommodityStore`].
    #[inline]
    pub fn commodity_store(&self) -> &CommodityStore<'ctx> {
        &self.commodities
    }
    /// Returns mut [`CommodityStore`].
    #[inline]
    pub fn commodity_store_mut(&mut self) -> &mut CommodityStore<'ctx> {
        &mut self.commodities
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn context_all_accounts() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let want = vec![
            ctx.accounts.ensure("Account 1"),
            ctx.accounts.ensure("Account 2"),
            ctx.accounts.ensure("Account 2:Sub account"),
            ctx.accounts.ensure("Account 3"),
            // I don't think ordering in Japanese doesn't make sense a lot without 'yomi' information.
            // OTOH, I don't have plan to use Japanese label, thus no urgent priorities.
            ctx.accounts.ensure("資産:りんご"),
            ctx.accounts.ensure("資産:バナナ"),
            ctx.accounts.ensure("資産:吉祥寺"),
        ];

        let got = ctx.all_accounts();

        assert_eq!(want, got);
    }

    #[test]
    fn context_sccount() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let a1 = ctx.accounts.ensure("Account 1");
        let a3 = ctx.accounts.ensure("Account 3");

        assert_eq!(Some(a1), ctx.account("Account 1"));
        assert_eq!(None, ctx.account("Account 2"));
        assert_eq!(Some(a3), ctx.account("Account 3"));
    }
}
