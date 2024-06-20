use bumpalo::Bump;

use super::intern;

/// ReportContext is a context object extensively used across Ledger evaluation.
pub struct ReportContext<'ctx> {
    pub(super) accounts: intern::AccountStore<'ctx>,
}

impl<'ctx> ReportContext<'ctx> {
    pub fn new(allocator: &'ctx Bump) -> Self {
        let accounts = intern::AccountStore::new(allocator);
        Self { accounts }
    }

    pub fn all_accounts(&'ctx self) -> Vec<intern::Account<'ctx>> {
        let mut r: Vec<intern::Account<'ctx>> = self.accounts.iter().collect();
        r.sort_unstable_by_key(|x| x.as_str());
        r
    }
}
