//! Provides query of transactions / balances on the processed [Ledger] instance.

use std::{borrow::Cow, collections::HashSet};

use chrono::NaiveDate;

use crate::{parse, syntax};

use super::{
    balance::Balance,
    commodity::OwnedCommodity,
    context::{Account, ReportContext},
    eval::{Amount, EvalError, Evaluable},
    price_db::{self, PriceRepository},
    transaction::{Posting, Transaction},
};

/// Contains processed transactions, so that users can query information.
pub struct Ledger<'ctx> {
    pub(super) transactions: Vec<Transaction<'ctx>>,
    pub(super) raw_balance: Balance<'ctx>,
    pub(super) price_repos: PriceRepository<'ctx>,
}

/// Error type for [`Ledger`] methods.
// TODO: Organize errors.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("failed to parse the given value")]
    ParseFailed(#[from] parse::ParseError),
    #[error("failed to evaluate the expr")]
    EvalFailed(#[from] EvalError),
    #[error("commodity {0} not found")]
    CommodityNotFound(OwnedCommodity),
    #[error("cannot convert amount: {0}")]
    CommodityConversionFailure(String),
}

/// Query to list postings matching the criteria.
#[derive(Debug)]
pub struct PostingQuery {
    /// Select the specified account if specified.
    /// Note this will be changed to list of regex eventually.
    pub account: Option<String>,
}

/// Context passed to [`Ledger::eval()`].
#[derive(Debug)]
pub struct EvalContext {
    pub date: NaiveDate,
    pub exchange: Option<String>,
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
    /// that will be added soon.
    pub fn balance(&self) -> Cow<'_, Balance<'ctx>> {
        Cow::Borrowed(&self.raw_balance)
    }

    /// Evals given `expression` with the given condition.
    pub fn eval(
        &mut self,
        ctx: &ReportContext<'ctx>,
        expression: &str,
        eval_ctx: &EvalContext,
    ) -> Result<Amount<'ctx>, QueryError> {
        let exchange = eval_ctx
            .exchange
            .as_ref()
            .map(|x| {
                ctx.commodities.resolve(&x).ok_or_else(|| {
                    QueryError::CommodityNotFound(OwnedCommodity::from_string(x.to_owned()))
                })
            })
            .transpose()?;
        let parsed: syntax::expr::ValueExpr = expression
            .try_into()
            .map_err(|err| QueryError::ParseFailed(err))?;
        let evaled: Amount<'ctx> = parsed.eval(ctx)?.try_into()?;
        let evaled = match exchange {
            None => evaled,
            Some(price_with) => {
                price_db::convert_amount(&mut self.price_repos, &evaled, price_with, eval_ctx.date)
                    .map_err(|err| QueryError::CommodityConversionFailure(err.to_string()))?
            }
        };
        Ok(evaled)
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use bumpalo::Bump;
    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::{load, report};

    fn fake_loader() -> load::Loader<load::FakeFileSystem> {
        let content = indoc! {"
            2024/01/01 Initial
                Assets:J 銀行               100,000 JPY
                Assets:CH Bank             2,000.00 CHF
                Liabilities:EUR Card        -300.00 EUR
                Equity
        "};
        let fake = hashmap! {
            PathBuf::from("/path/to/file.ledger") => content.as_bytes().to_vec(),
        };
        load::Loader::new(PathBuf::from("/path/to/file.ledger"), fake.into())
    }

    fn create_ledger<'ctx>(
        arena: &'ctx Bump,
    ) -> Result<(ReportContext<'ctx>, Ledger<'ctx>), report::ReportError> {
        let mut ctx = ReportContext::new(&arena);
        let ledger = report::process(&mut ctx, fake_loader(), &report::ProcessOptions::default())?;
        Ok((ctx, ledger))
    }

    #[test]
    fn balance_default() {
        let arena = Bump::new();
        let (ctx, ledger) = create_ledger(&arena).unwrap();

        let got = ledger.balance();

        log::info!("all_accounts: {:?}", ctx.all_accounts());

        let want: Balance = [
            (
                ctx.account("Assets:J 銀行").unwrap(),
                Amount::from_value(dec!(100000), ctx.commodities.resolve("JPY").unwrap()),
            ),
            (
                ctx.account("Assets:CH Bank").unwrap(),
                Amount::from_value(dec!(2000.00), ctx.commodities.resolve("CHF").unwrap()),
            ),
            (
                ctx.account("Liabilities:EUR Card").unwrap(),
                Amount::from_value(dec!(-300.00), ctx.commodities.resolve("EUR").unwrap()),
            ),
            (
                ctx.account("Equity").unwrap(),
                Amount::from_values([
                    (dec!(-100000), ctx.commodities.resolve("JPY").unwrap()),
                    (dec!(-2000.00), ctx.commodities.resolve("CHF").unwrap()),
                    (dec!(300.00), ctx.commodities.resolve("EUR").unwrap()),
                ]),
            ),
        ]
        .into_iter()
        .collect();
        assert_eq!(Cow::Borrowed(&want), got);
    }

    #[test]
    fn eval_default_context() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena).unwrap();

        let evaluated = ledger
            .eval(
                &ctx,
                "1 JPY",
                &EvalContext {
                    date: NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
                    exchange: None,
                },
            )
            .unwrap();

        assert_eq!(
            Amount::from_value(dec!(1), ctx.commodities.resolve("JPY").unwrap()),
            evaluated
        );
    }
}
