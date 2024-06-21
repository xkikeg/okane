use bumpalo::Bump;

use super::intern;

/// Context object extensively used across Ledger file evaluation.
pub struct ReportContext<'ctx> {
    pub(super) arena: &'ctx Bump,
    pub(super) accounts: intern::AccountStore<'ctx>,
    pub(super) commodities: intern::CommodityStore<'ctx>,
}

impl<'ctx> ReportContext<'ctx> {
    /// Create a new instance of `ReportContext`.
    pub fn new(arena: &'ctx Bump) -> Self {
        let accounts = intern::AccountStore::new(arena);
        let commodities = intern::CommodityStore::new(arena);
        Self {
            arena,
            accounts,
            commodities,
        }
    }

    /// Returns all accounts, sorted as string order.
    pub fn all_accounts(&'ctx self) -> Vec<intern::Account<'ctx>> {
        let mut r: Vec<intern::Account<'ctx>> = self.accounts.iter().collect();
        r.sort_unstable_by_key(|x| x.as_str());
        r
    }

    /// Returns the given account, or `None` if not found.
    pub fn account(&'ctx self, value: &str) -> Option<intern::Account<'ctx>> {
        self.accounts.get(value)
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
            ctx.accounts.intern("Account 1"),
            ctx.accounts.intern("Account 2"),
            ctx.accounts.intern("Account 2:Sub account"),
            ctx.accounts.intern("Account 3"),
            // I don't think ordering in Japanese doesn't make sense a lot without 'yomi' information.
            // OTOH, I don't have plan to use Japanese label, thus no urgent priorities.
            ctx.accounts.intern("資産:りんご"),
            ctx.accounts.intern("資産:バナナ"),
            ctx.accounts.intern("資産:吉祥寺"),
        ];

        let got = ctx.all_accounts();

        assert_eq!(want, got);
    }

    #[test]
    fn context_sccount() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let a1 = ctx.accounts.intern("Account 1");
        let a3 = ctx.accounts.intern("Account 3");

        assert_eq!(Some(a1), ctx.account("Account 1"));
        assert_eq!(None, ctx.account("Account 2"));
        assert_eq!(Some(a3), ctx.account("Account 3"));
    }
}
