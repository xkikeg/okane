use std::collections::HashMap;

use super::{context::Account, eval::Amount};

/// Accumulated balance of accounts.
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

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::super::context::ReportContext;

    #[test]
    fn balance_gives_zero_amount_when_not_initalized() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let balance = Balance::default();
        assert_eq!(balance.get_balance(&ctx.accounts.ensure("Expenses")), None);
    }

    #[test]
    fn test_balance_increment_adds_value() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let mut balance = Balance::default();
        let updated = balance.increment(
            ctx.accounts.ensure("Expenses"),
            Amount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );

        assert_eq!(
            updated,
            Amount::from_value(dec!(1000), ctx.commodities.ensure("JPY"))
        );
        assert_eq!(
            balance.get_balance(&ctx.accounts.ensure("Expenses")),
            Some(&updated)
        );

        let updated = balance.increment(
            ctx.accounts.ensure("Expenses"),
            Amount::from_value(dec!(-1000), ctx.commodities.ensure("JPY")),
        );

        assert_eq!(updated, Amount::zero());
        assert_eq!(
            balance.get_balance(&ctx.accounts.ensure("Expenses")),
            Some(&updated)
        );
    }

    #[test]
    fn test_balance_set_balance() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let mut balance = Balance::default();
        let prev = balance.set_balance(
            ctx.accounts.ensure("Expenses"),
            Amount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );

        assert_eq!(prev, Amount::zero());
        assert_eq!(
            balance.get_balance(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_value(
                dec!(1000),
                ctx.commodities.ensure("JPY")
            ))
        );

        let prev = balance.set_balance(
            ctx.accounts.ensure("Expenses"),
            Amount::from_value(dec!(-1000), ctx.commodities.ensure("JPY")),
        );

        assert_eq!(
            prev,
            Amount::from_value(dec!(1000), ctx.commodities.ensure("JPY"))
        );
        assert_eq!(
            balance.get_balance(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_value(
                dec!(-1000),
                ctx.commodities.ensure("JPY")
            ))
        );
    }
}
