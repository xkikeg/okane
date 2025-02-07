use std::collections::HashMap;

use super::{
    context::Account,
    eval::{Amount, EvalError, PostingAmount},
    ReportContext,
};

/// Error related to [Balance] operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BalanceError {
    #[error("balance = 0 cannot deduce posting amount when balance has multi commodities")]
    MultiCommodityWithPartialSet(#[from] EvalError),
}

/// Accumulated balance of accounts.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Balance<'ctx> {
    accounts: HashMap<Account<'ctx>, Amount<'ctx>>,
}

impl<'ctx> FromIterator<(Account<'ctx>, Amount<'ctx>)> for Balance<'ctx> {
    /// Constructs [Balance] instance out of Iterator.
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Account<'ctx>, Amount<'ctx>)>,
    {
        Self {
            accounts: iter.into_iter().collect(),
        }
    }
}

impl<'ctx> Balance<'ctx> {
    /// Adds a particular account value, and returns the updated balance.
    pub fn add_amount(&mut self, account: Account<'ctx>, amount: Amount<'ctx>) -> &Amount<'ctx> {
        let curr: &mut Amount = self.accounts.entry(account).or_default();
        *curr += amount;
        curr.remove_zero_entries();
        curr
    }

    /// Adds a particular account value with the specified commodity, and returns the updated balance.
    pub(super) fn add_posting_amount(
        &mut self,
        account: Account<'ctx>,
        amount: PostingAmount<'ctx>,
    ) -> &Amount<'ctx> {
        let curr: &mut Amount = self.accounts.entry(account).or_default();
        *curr += amount;
        curr.remove_zero_entries();
        curr
    }

    /// Tries to set the particular account's balance with the specified commodity,
    /// and returns the delta which should have caused the difference.
    pub(super) fn set_partial(
        &mut self,
        account: Account<'ctx>,
        amount: PostingAmount<'ctx>,
    ) -> Result<PostingAmount<'ctx>, BalanceError> {
        match amount {
            PostingAmount::Zero => {
                let prev: Amount<'ctx> = self
                    .accounts
                    .insert(account, Amount::zero())
                    .unwrap_or_default();
                (&prev)
                    .try_into()
                    .map_err(BalanceError::MultiCommodityWithPartialSet)
            }
            PostingAmount::Single(single_amount) => {
                let prev = self
                    .accounts
                    .entry(account)
                    .or_default()
                    .set_partial(single_amount);
                Ok(PostingAmount::Single(prev))
            }
        }
    }

    /// Gets the balance of the given account.
    pub fn get(&self, account: &Account<'ctx>) -> Option<&Amount<'ctx>> {
        self.accounts.get(account)
    }

    /// Rounds the balance following the context.
    pub fn round(&mut self, ctx: &ReportContext<'ctx>) {
        for amount in self.accounts.values_mut() {
            amount.round_mut(ctx);
        }
    }

    /// Returns unordered iterator for the account and the amount.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Account<'ctx>, &Amount<'ctx>)> {
        self.accounts.iter()
    }

    /// Constructs sorted vec of account and commodity tuple.
    pub fn into_vec(self) -> Vec<(Account<'ctx>, Amount<'ctx>)> {
        let mut ret: Vec<(Account<'ctx>, Amount<'ctx>)> = self.accounts.into_iter().collect();
        ret.sort_unstable_by_key(|(a, _)| a.as_str());
        ret
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
        assert_eq!(balance.get(&ctx.accounts.ensure("Expenses")), None);
    }

    #[test]
    fn test_balance_increment_adds_value() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let mut balance = Balance::default();
        let updated = balance
            .add_posting_amount(
                ctx.accounts.ensure("Expenses"),
                PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
            )
            .clone();

        assert_eq!(
            updated,
            Amount::from_value(dec!(1000), ctx.commodities.ensure("JPY"))
        );
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&updated)
        );

        let updated = balance
            .add_posting_amount(
                ctx.accounts.ensure("Expenses"),
                PostingAmount::from_value(dec!(-1000), ctx.commodities.ensure("JPY")),
            )
            .clone();

        assert_eq!(updated, Amount::zero());
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&updated)
        );
    }

    #[test]
    fn test_balance_set_partial_from_absolute_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();

        let prev = balance
            .set_partial(
                ctx.accounts.ensure("Expenses"),
                PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
            )
            .unwrap();

        // Note it won't be PostingAmount::zero(),
        // as set_partial is called with commodity amount.
        assert_eq!(
            prev,
            PostingAmount::from_value(dec!(0), ctx.commodities.ensure("JPY"))
        );
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_value(
                dec!(1000),
                ctx.commodities.ensure("JPY")
            ))
        );
    }

    #[test]
    fn test_balance_set_partial_hit_same_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );

        let prev = balance
            .set_partial(
                ctx.accounts.ensure("Expenses"),
                PostingAmount::from_value(dec!(-1000), ctx.commodities.ensure("JPY")),
            )
            .unwrap();

        assert_eq!(
            prev,
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY"))
        );
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_value(
                dec!(-1000),
                ctx.commodities.ensure("JPY")
            ))
        );
    }

    #[test]
    fn test_balance_set_partial_multi_commodities() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(dec!(200), ctx.commodities.ensure("CHF")),
        );

        let prev = balance
            .set_partial(
                ctx.accounts.ensure("Expenses"),
                PostingAmount::from_value(dec!(100), ctx.commodities.ensure("CHF")),
            )
            .unwrap();

        assert_eq!(
            prev,
            PostingAmount::from_value(dec!(200), ctx.commodities.ensure("CHF"))
        );
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_values([
                (dec!(1000), ctx.commodities.ensure("JPY")),
                (dec!(100), ctx.commodities.ensure("CHF")),
            ]))
        );
    }

    #[test]
    fn test_balance_set_partial_zero_on_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();

        let prev = balance
            .set_partial(ctx.accounts.ensure("Expenses"), PostingAmount::zero())
            .unwrap();

        assert_eq!(prev, PostingAmount::zero());
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::zero())
        );
    }

    #[test]
    fn test_balance_set_partial_zero_on_single_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );

        let prev = balance
            .set_partial(ctx.accounts.ensure("Expenses"), PostingAmount::zero())
            .unwrap();

        assert_eq!(
            prev,
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY"))
        );
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::zero())
        );
    }

    #[test]
    fn test_balance_set_partial_zero_fails_on_multi_commodities() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(dec!(200), ctx.commodities.ensure("CHF")),
        );

        let err = balance
            .set_partial(ctx.accounts.ensure("Expenses"), PostingAmount::zero())
            .unwrap_err();

        assert_eq!(
            err,
            BalanceError::MultiCommodityWithPartialSet(EvalError::PostingAmountRequired)
        );
    }
}
