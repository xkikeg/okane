//! Provides query of transactions / balances on the processed [Ledger] instance.

use std::{borrow::Cow, collections::HashSet};

use chrono::NaiveDate;

use crate::{
    parse,
    report::{commodity::CommodityTag, eval::OwnedEvalError},
    syntax,
};

use super::{
    balance::Balance,
    commodity::OwnedCommodity,
    context::{Account, ReportContext},
    eval::{Amount, EvalError, Evaluable},
    price_db::{self, PriceRepository},
    transaction::{Posting, Transaction},
};

/// Contains processed transactions, so that users can query information.
#[derive(Debug)]
pub struct Ledger<'ctx> {
    pub(super) transactions: Vec<Transaction<'ctx>>,
    pub(super) raw_balance: Balance<'ctx>,
    pub(super) price_repos: PriceRepository<'ctx>,
}

/// Error type for [`Ledger`] methods.
// TODO: Organize errors.
// TODO: non_exhaustive
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("failed to parse the given value")]
    ParseFailed(#[from] parse::ParseError),
    #[error("failed to evaluate the expr")]
    EvalFailed(#[from] OwnedEvalError),
    #[error("commodity {0} not found")]
    CommodityNotFound(OwnedCommodity),
    #[error("cannot convert amount: {0}")]
    CommodityConversionFailure(String),
}

/// Query to list postings matching the criteria.
// TODO: non_exhaustive
#[derive(Debug)]
pub struct PostingQuery {
    /// Select the specified account if specified.
    /// Note this will be changed to list of regex eventually.
    pub account: Option<String>,
}

/// Specifies the conversion strategy.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConversionStrategy {
    /// Converts the amount on the date of transaction.
    Historical,
    /// Converts the amount on the given date's rate.
    /// Not implemented for register report.
    /// https://github.com/xkikeg/okane/issues/313
    UpToDate {
        /// Date for the conversion to be _up-to-date_.
        today: NaiveDate,
    },
}

/// Instruction about commodity conversion.
#[derive(Debug)]
// TODO: non_exhaustive
pub struct Conversion<'ctx> {
    pub strategy: ConversionStrategy,
    pub target: CommodityTag<'ctx>,
}

/// Half-open range of the date for the query result.
/// If any of `start` or `end` is set as [`None`],
/// those are treated as -infinity, +infinity respectively.
#[derive(Debug, Default)]
pub struct DateRange {
    /// Start of the range (inclusive), if exists.
    pub start: Option<NaiveDate>,
    /// End of the range (exclusive), if exists.
    pub end: Option<NaiveDate>,
}

impl DateRange {
    fn is_bypass(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    fn contains(&self, date: NaiveDate) -> bool {
        match (self.start, self.end) {
            (Some(start), _) if date < start => false,
            (_, Some(end)) if end <= date => false,
            _ => true,
        }
    }
}

/// Query for [`Ledger::balance()`].
#[derive(Debug, Default)]
// TODO: non_exhaustive
pub struct BalanceQuery<'ctx> {
    pub conversion: Option<Conversion<'ctx>>,
    pub date_range: DateRange,
}

impl BalanceQuery<'_> {
    fn require_recompute(&self) -> bool {
        if !self.date_range.is_bypass() {
            return true;
        }
        if matches!(&self.conversion, Some(conv) if conv.strategy == ConversionStrategy::Historical)
        {
            return true;
        }
        // at this case, we can reuse the balance for the whole range data.
        false
    }
}

/// Context passed to [`Ledger::eval()`].
#[derive(Debug)]
// TODO: non_exhaustive
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
    // TODO: Support date range query.
    // https://github.com/xkikeg/okane/issues/208
    // TODO: Support currency conversion.
    // https://github.com/xkikeg/okane/issues/313
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
    pub fn balance(
        &mut self,
        ctx: &ReportContext<'ctx>,
        query: &BalanceQuery<'ctx>,
    ) -> Result<Cow<'_, Balance<'ctx>>, QueryError> {
        let balance = if !query.require_recompute() {
            Cow::Borrowed(&self.raw_balance)
        } else {
            let mut bal = Balance::default();
            for (txn, posting) in self.transactions.iter().flat_map(|txn| {
                txn.postings.iter().filter_map(move |posting| {
                    if !query.date_range.contains(txn.date) {
                        return None;
                    }
                    Some((txn, posting))
                })
            }) {
                let delta = match query.conversion {
                    Some(Conversion {
                        strategy: ConversionStrategy::Historical,
                        target,
                    }) => Cow::Owned(
                        price_db::convert_amount(
                            ctx,
                            &mut self.price_repos,
                            &posting.amount,
                            target,
                            txn.date,
                        )
                        // TODO: do we need this round, or just at the end?
                        // .map(|amount| amount.round(ctx))
                        .map_err(|err| QueryError::CommodityConversionFailure(err.to_string()))?,
                    ),
                    None
                    | Some(Conversion {
                        strategy: ConversionStrategy::UpToDate { .. },
                        ..
                    }) => Cow::Borrowed(&posting.amount),
                };
                bal.add_amount(posting.account, delta.into_owned());
            }
            bal.round(ctx);
            Cow::Owned(bal)
        };
        match query.conversion {
            None
            | Some(Conversion {
                strategy: ConversionStrategy::Historical,
                ..
            }) => Ok(balance),
            Some(Conversion {
                strategy: ConversionStrategy::UpToDate { today },
                target,
            }) => {
                let mut converted = Balance::default();
                for (account, original_amount) in balance.iter() {
                    converted.add_amount(
                        *account,
                        price_db::convert_amount(
                            ctx,
                            &mut self.price_repos,
                            original_amount,
                            target,
                            today,
                        )
                        .map_err(|err| QueryError::CommodityConversionFailure(err.to_string()))?,
                    );
                }
                converted.round(ctx);
                Ok(Cow::Owned(converted))
            }
        }
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
                ctx.commodities.resolve(x).ok_or_else(|| {
                    QueryError::CommodityNotFound(OwnedCommodity::from_string(x.to_owned()))
                })
            })
            .transpose()?;
        let parsed: syntax::expr::ValueExpr =
            expression.try_into().map_err(QueryError::ParseFailed)?;
        let evaled: Amount<'ctx> = parsed
            .eval(ctx)
            .map_err(|e| e.into_owned(ctx))?
            .try_into()
            .map_err(|e: EvalError<'_>| e.into_owned(ctx))?;
        let evaled = match exchange {
            None => evaled,
            Some(price_with) => price_db::convert_amount(
                ctx,
                &mut self.price_repos,
                &evaled,
                price_with,
                eval_ctx.date,
            )
            .map_err(|err| QueryError::CommodityConversionFailure(err.to_string()))?,
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

    use crate::{load, report, testing::recursive_print};

    fn fake_loader() -> load::Loader<load::FakeFileSystem> {
        let content = indoc! {"
            commodity JPY
                format 1,000 JPY

            2023/12/31 rate
                Equity                         0.00 CHF @ 168.24 JPY
                Equity                         0.00 EUR @ 157.12 JPY

            2024/01/01 Initial
                Assets:J 銀行             1,000,000 JPY
                Assets:CH Bank             2,000.00 CHF
                Liabilities:EUR Card        -300.00 EUR
                Assets:Broker            5,000.0000 OKANE {80 JPY}
                Equity

            2024/01/05 Shopping
                Expenses:Grocery             100.00 CHF @ 171.50 JPY
                Assets:J 銀行               -17,150 JPY

            2024/01/09 Buy Stock
                Assets:Broker               23.0000 OKANE {120 JPY}
                Assets:J 銀行                -2,760 JPY

            2024/01/15 Sell Stock
                Assets:Broker                12,300 JPY
                Assets:Broker             -100.0000 OKANE {80 JPY} @ 100 JPY
                Assets:Broker              -23.0000 OKANE {120 JPY} @ 100 JPY
                Income:Capital Gain           -1540 JPY

            2024/01/20 Shopping
                Expenses:Food                150.00 EUR @ 0.9464 CHF
                Assets:CH Bank              -141.96 CHF
        "};
        let fake = hashmap! {
            PathBuf::from("path/to/file.ledger") => content.as_bytes().to_vec(),
        };
        load::Loader::new(PathBuf::from("path/to/file.ledger"), fake.into())
    }

    fn create_ledger(arena: &Bump) -> (ReportContext<'_>, Ledger<'_>) {
        let mut ctx = ReportContext::new(arena);
        let ledger =
            match report::process(&mut ctx, fake_loader(), &report::ProcessOptions::default()) {
                Ok(v) => v,
                Err(err) => panic!(
                    "failed to create the testing Ledger: {}",
                    recursive_print(err),
                ),
            };
        (ctx, ledger)
    }

    #[test]
    fn balance_default() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

        log::info!("all_accounts: {:?}", ctx.all_accounts());
        let chf = ctx.commodities.resolve("CHF").unwrap();
        let eur = ctx.commodities.resolve("EUR").unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let okane = ctx.commodities.resolve("OKANE").unwrap();

        let got = ledger.balance(&ctx, &BalanceQuery::default()).unwrap();

        let want: Balance = [
            (
                ctx.account("Assets:CH Bank").unwrap(),
                Amount::from_value(chf, dec!(1858.04)),
            ),
            (
                ctx.account("Assets:Broker").unwrap(),
                Amount::from_iter([(okane, dec!(4900.0000)), (jpy, dec!(12300))]),
            ),
            (
                ctx.account("Assets:J 銀行").unwrap(),
                Amount::from_value(jpy, dec!(980090)),
            ),
            (
                ctx.account("Liabilities:EUR Card").unwrap(),
                Amount::from_value(eur, dec!(-300.00)),
            ),
            (
                ctx.account("Income:Capital Gain").unwrap(),
                Amount::from_value(jpy, dec!(-1540)),
            ),
            (
                ctx.account("Expenses:Food").unwrap(),
                Amount::from_value(eur, dec!(150.00)),
            ),
            (
                ctx.account("Expenses:Grocery").unwrap(),
                Amount::from_value(chf, dec!(100.00)),
            ),
            (
                ctx.account("Equity").unwrap(),
                Amount::from_iter([
                    (jpy, dec!(-1_400_000)),
                    (chf, dec!(-2000.00)),
                    (eur, dec!(300.00)),
                ]),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(want.into_vec(), got.into_owned().into_vec());
    }

    #[test]
    fn balance_conversion_historical() {
        // actually it doesn't quite make sense to have balance with
        // either --historical or --up-to-date,
        // because up-to-date makes sense for Assets & Liabilities
        // while historical does for Income / Expenses.
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

        let jpy = ctx.commodities.resolve("JPY").unwrap();

        let got = ledger
            .balance(
                &ctx,
                &BalanceQuery {
                    conversion: Some(Conversion {
                        strategy: ConversionStrategy::Historical,
                        target: jpy,
                    }),
                    date_range: DateRange::default(),
                },
            )
            .unwrap();

        let want: Balance = [
            (
                ctx.account("Assets:CH Bank").unwrap(),
                // 2000.00 * 168.24 - 141.96 * 171.50
                Amount::from_value(jpy, dec!(312_134)),
            ),
            (
                ctx.account("Assets:Broker").unwrap(),
                // 5_000 * 80 = 400_000
                // 23 * 120 = 2_760
                // -100 * 100 = -10_000
                // -23 * 100 = -2_300
                Amount::from_iter([
                    (jpy, dec!(400_000)),
                    (jpy, dec!(2_760)),
                    (jpy, dec!(-10_000)),
                    (jpy, dec!(-2_300)),
                    (jpy, dec!(12_300)),
                ]),
            ),
            (
                ctx.account("Assets:J 銀行").unwrap(),
                Amount::from_value(jpy, dec!(980090)),
            ),
            (
                ctx.account("Liabilities:EUR Card").unwrap(),
                Amount::from_value(jpy, dec!(-47_136)),
            ),
            (
                ctx.account("Income:Capital Gain").unwrap(),
                Amount::from_value(jpy, dec!(-1_540)),
            ),
            (
                ctx.account("Expenses:Grocery").unwrap(),
                Amount::from_value(jpy, dec!(17_150)),
            ),
            (
                ctx.account("Expenses:Food").unwrap(),
                Amount::from_value(jpy, dec!(23_568)),
            ),
            (
                ctx.account("Equity").unwrap(),
                Amount::from_iter([
                    (jpy, dec!(-1_400_000)),
                    // -2000 * 168.24, historical rate.
                    (jpy, dec!(-336_480)),
                    // 300 * 157.12
                    (jpy, dec!(47_136)),
                ]),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(want.into_vec(), got.into_owned().into_vec());
    }

    #[test]
    fn balance_conversion_up_to_date() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

        let jpy = ctx.commodities.resolve("JPY").unwrap();

        let got = ledger
            .balance(
                &ctx,
                &BalanceQuery {
                    conversion: Some(Conversion {
                        strategy: ConversionStrategy::UpToDate {
                            today: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                        },
                        target: jpy,
                    }),
                    date_range: DateRange::default(),
                },
            )
            .unwrap();

        let want: Balance = [
            (
                ctx.account("Assets:CH Bank").unwrap(),
                // 1858.04 * 171.50
                Amount::from_value(jpy, dec!(318_654)),
            ),
            (
                ctx.account("Assets:Broker").unwrap(),
                Amount::from_value(jpy, dec!(502_300)),
            ),
            (
                ctx.account("Assets:J 銀行").unwrap(),
                Amount::from_value(jpy, dec!(980090)),
            ),
            (
                ctx.account("Liabilities:EUR Card").unwrap(),
                // -300 * 157.12, EUR/JPY rate won't use EUR/CHF CHF/JPY.
                Amount::from_value(jpy, dec!(-47_136)),
            ),
            (
                ctx.account("Income:Capital Gain").unwrap(),
                Amount::from_value(jpy, dec!(-1540)),
            ),
            (
                ctx.account("Expenses:Food").unwrap(),
                // 150.00 EUR * 157.12
                Amount::from_value(jpy, dec!(23_568)),
            ),
            (
                ctx.account("Expenses:Grocery").unwrap(),
                Amount::from_value(jpy, dec!(17_150)),
            ),
            (
                ctx.account("Equity").unwrap(),
                Amount::from_iter([
                    (jpy, dec!(-1_400_000)),
                    // -2000 CHF * 171.50
                    (jpy, dec!(-343_000)),
                    // 300 EUR * 157.12
                    (jpy, dec!(47_136)),
                ]),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(want.into_vec(), got.into_owned().into_vec());
    }

    #[test]
    fn balance_date_range() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

        log::info!("all_accounts: {:?}", ctx.all_accounts());
        let chf = ctx.commodities.resolve("CHF").unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();

        let got = ledger
            .balance(
                &ctx,
                &BalanceQuery {
                    conversion: None,
                    date_range: DateRange {
                        start: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
                        end: Some(NaiveDate::from_ymd_opt(2024, 1, 9).unwrap()),
                    },
                },
            )
            .unwrap();

        let want: Balance = [
            (
                ctx.account("Assets:J 銀行").unwrap(),
                Amount::from_value(jpy, dec!(-17150)),
            ),
            (
                ctx.account("Expenses:Grocery").unwrap(),
                Amount::from_value(chf, dec!(100.00)),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(want.into_vec(), got.into_owned().into_vec());
    }

    #[test]
    fn eval_default_context() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

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
            Amount::from_value(ctx.commodities.resolve("JPY").unwrap(), dec!(1)),
            evaluated
        );
    }
}
