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
