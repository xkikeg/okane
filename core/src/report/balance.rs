use std::collections::HashMap;

use crate::report::eval::EvalError;

use super::{
    context::Account,
    eval::{Amount, OwnedEvalError, PostingAmount},
    ReportContext,
};

/// Error related to [Balance] operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BalanceError {
    #[error("balance = 0 should be used on single commodity balance")]
    MultiCommodityWithPartialSet(#[source] OwnedEvalError, String),
}

impl BalanceError {
    pub(super) fn note(&self) -> impl std::fmt::Display + '_ {
        BalanceErrorNote(self)
    }
}

struct BalanceErrorNote<'a>(&'a BalanceError);

impl std::fmt::Display for BalanceErrorNote<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BalanceError::MultiCommodityWithPartialSet(_, balance) => {
                write!(f, "actual: {balance}")
            }
        }
    }
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
        ctx: &ReportContext<'ctx>,
        account: Account<'ctx>,
        amount: PostingAmount<'ctx>,
    ) -> Result<PostingAmount<'ctx>, BalanceError> {
        match amount {
            PostingAmount::Zero => {
                let prev: Amount<'ctx> = self
                    .accounts
                    .insert(account, Amount::zero())
                    .unwrap_or_default();
                (&prev).try_into().map_err(|e: EvalError<'_>| {
                    BalanceError::MultiCommodityWithPartialSet(
                        e.into_owned(ctx),
                        prev.as_inline_display(ctx).to_string(),
                    )
                })
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
                PostingAmount::from_value(ctx.commodities.ensure("JPY"), dec!(1000)),
            )
            .clone();

        assert_eq!(
            updated,
            Amount::from_value(ctx.commodities.ensure("JPY"), dec!(1000))
        );
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&updated)
        );

        let updated = balance
            .add_posting_amount(
                ctx.accounts.ensure("Expenses"),
                PostingAmount::from_value(ctx.commodities.ensure("JPY"), dec!(-1000)),
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

        let expenses = ctx.accounts.ensure("Expenses");
        let jpy = ctx.commodities.insert("JPY").unwrap();
        let prev = balance
            .set_partial(&ctx, expenses, PostingAmount::from_value(jpy, dec!(1000)))
            .unwrap();

        // Note it won't be PostingAmount::zero(),
        // as set_partial is called with commodity amount.
        assert_eq!(prev, PostingAmount::from_value(jpy, dec!(0)));
        assert_eq!(
            balance.get(&expenses),
            Some(&Amount::from_value(jpy, dec!(1000)))
        );
    }

    #[test]
    fn test_balance_set_partial_hit_same_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();
        let jpy = ctx.commodities.ensure("JPY");
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(jpy, dec!(1000)),
        );

        let expenses = ctx.accounts.ensure("Expenses");

        let prev = balance
            .set_partial(&ctx, expenses, PostingAmount::from_value(jpy, dec!(-1000)))
            .unwrap();

        assert_eq!(prev, PostingAmount::from_value(jpy, dec!(1000)));
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_value(jpy, dec!(-1000)))
        );
    }

    #[test]
    fn test_balance_set_partial_multi_commodities() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();
        let jpy = ctx.commodities.ensure("JPY");
        let chf = ctx.commodities.ensure("CHF");
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(jpy, dec!(1000)),
        );
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(chf, dec!(200)),
        );

        let expenses = ctx.accounts.ensure("Expenses");

        let prev = balance
            .set_partial(&ctx, expenses, PostingAmount::from_value(chf, dec!(100)))
            .unwrap();

        assert_eq!(prev, PostingAmount::from_value(chf, dec!(200)));
        assert_eq!(
            balance.get(&ctx.accounts.ensure("Expenses")),
            Some(&Amount::from_values([(jpy, dec!(1000)), (chf, dec!(100)),]))
        );
    }

    #[test]
    fn test_balance_set_partial_zero_on_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut balance = Balance::default();

        let expenses = ctx.accounts.ensure("Expenses");

        let prev = balance
            .set_partial(&ctx, expenses, PostingAmount::zero())
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
        let jpy = ctx.commodities.ensure("JPY");
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(jpy, dec!(1000)),
        );

        let expenses = ctx.accounts.ensure("Expenses");

        let prev = balance
            .set_partial(&ctx, expenses, PostingAmount::zero())
            .unwrap();

        assert_eq!(prev, PostingAmount::from_value(jpy, dec!(1000)));
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
            PostingAmount::from_value(ctx.commodities.ensure("JPY"), dec!(1000)),
        );
        balance.add_posting_amount(
            ctx.accounts.ensure("Expenses"),
            PostingAmount::from_value(ctx.commodities.ensure("CHF"), dec!(200)),
        );

        let expenses = ctx.accounts.ensure("Expenses");

        let err = balance
            .set_partial(&ctx, expenses, PostingAmount::zero())
            .unwrap_err();

        assert_eq!(
            err,
            BalanceError::MultiCommodityWithPartialSet(
                OwnedEvalError::PostingAmountRequired,
                "(1000 JPY + 200 CHF)".to_string()
            )
        );
    }
}
