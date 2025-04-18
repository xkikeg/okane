use bumpalo::Bump;

use super::{
    commodity::{Commodity, CommodityStore},
    intern::{FromInterned, InternStore, InternedStr, StoredValue},
};

/// `&str` for accounts, interned within the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Account<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for Account<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }

    fn as_interned(&self) -> InternedStr<'arena> {
        self.0
    }
}

impl<'arena> Account<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// `Interner` for `Account`.
pub(super) type AccountStore<'arena> = InternStore<'arena, Account<'arena>>;

/// Context object extensively used across Ledger file evaluation.
pub struct ReportContext<'ctx> {
    pub(super) arena: &'ctx Bump,
    pub(super) accounts: AccountStore<'ctx>,
    pub(super) commodities: CommodityStore<'ctx>,
}

impl<'ctx> ReportContext<'ctx> {
    /// Create a new instance of `ReportContext`.
    pub fn new(arena: &'ctx Bump) -> Self {
        let accounts = AccountStore::new(arena);
        let commodities = CommodityStore::new(arena);
        Self {
            arena,
            accounts,
            commodities,
        }
    }

    /// Returns all accounts, sorted as string order.
    pub(super) fn all_accounts_unsorted(&self) -> impl Iterator<Item = Account<'ctx>> + '_ {
        self.accounts.iter().filter_map(|x| match x {
            StoredValue::Canonical(x) => Some(x),
            StoredValue::Alias { .. } => None,
        })
    }
    /// Returns all accounts, sorted as string order.
    pub(super) fn all_accounts(&self) -> Vec<Account<'ctx>> {
        let mut r: Vec<Account<'ctx>> = self.all_accounts_unsorted().collect();
        r.sort_unstable_by_key(|x| x.as_str());
        r
    }

    /// Returns the given account, or `None` if not found.
    pub fn account(&self, value: &str) -> Option<Account<'ctx>> {
        self.accounts.resolve(value)
    }

    /// Returns the given commmodity, or `None` if not found.
    pub fn commodity(&self, value: &str) -> Option<Commodity<'ctx>> {
        self.commodities.resolve(value)
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
