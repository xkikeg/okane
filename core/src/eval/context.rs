use crate::eval::types;

use bumpalo::Bump;

/// EvalContext is a context object extensively used across Ledger evaluation.
pub struct EvalContext<'ctx> {
    pub(super) accounts: types::AccountStore<'ctx>,
    pub(super) commodities: types::CommodityStore<'ctx>,
}

impl<'ctx> EvalContext<'ctx> {
    pub fn new(allocator: &'ctx Bump) -> Self {
        let accounts = types::AccountStore::new(allocator);
        let commodities = types::CommodityStore::new(allocator);
        Self {
            accounts,
            commodities,
        }
    }

    pub fn all_accounts(&'ctx self) -> Vec<types::Account<'ctx>> {
        let mut r: Vec<types::Account<'ctx>> = self.accounts.iter().collect();
        r.sort_unstable_by_key(|x| x.as_str());
        r
    }
}
