//! Provides query of transactions / balances on the processed [Ledger] instance.

use std::{borrow::Cow, collections::HashSet};

use chrono::NaiveDate;
use lender::{check_covariance_fallible, FallibleLender, FallibleLending};

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
    price_db::{self, ConversionError, PriceRepository},
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
    CommodityConversionFailure(#[from] ConversionError),
    #[error("unsupported conversion strategy for this query")]
    UnsupportedConversionStrategy,
}

/// Query to list postings matching the criteria.
// TODO: non_exhaustive
#[derive(Debug)]
pub struct PostingQuery {
    /// Select the specified account if specified.
    /// Note this will be changed to list of regex eventually.
    pub account: Option<String>,
}

/// Query to drive [`Ledger::register_entries`].
#[derive(Debug, Default)]
// TODO: non_exhaustive
pub struct RegisterQuery<'ctx> {
    /// Select the specified account if specified.
    /// Note this will be changed to a list of regex eventually.
    pub account: Option<String>,
    /// Half-open date range to restrict transactions.
    pub date_range: DateRange,
    /// Optional currency conversion applied to every yielded amount and to the
    /// running total. Only [`ConversionStrategy::Historical`] is supported.
    /// See https://github.com/xkikeg/okane/issues/313.
    pub conversion: Option<Conversion<'ctx>>,
}

/// A row of the register report.
///
/// Yielded by [`RegisterEntries`]. `total` is the running cumulative amount
/// across all matched postings up to and including this one; the borrow is
/// invalidated by the next call to [`Lender::next`].
#[derive(Debug)]
#[non_exhaustive]
pub struct RegisterEntry<'lend, 'ctx> {
    /// Date of the enclosing transaction.
    pub date: NaiveDate,
    /// Payee of the posting (per-posting `; Payee:` override falls back to the
    /// enclosing transaction's payee).
    pub payee: &'ctx str,
    /// Account of the matched posting.
    pub account: Account<'ctx>,
    /// Amount of the matched posting. If conversion is configured this is the
    /// converted amount.
    pub amount: &'lend Amount<'ctx>,
    /// Running cumulative amount across all matched postings up to and
    /// including this row.
    pub total: &'lend Amount<'ctx>,
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
#[derive(Debug, Clone, Copy)]
// TODO: non_exhaustive
pub struct Conversion<'ctx> {
    pub strategy: ConversionStrategy,
    pub target: CommodityTag<'ctx>,
}

/// Half-open range of the date for the query result.
/// If any of `start` or `end` is set as [`None`],
/// those are treated as -infinity, +infinity respectively.
#[derive(Debug, Default, Clone, Copy)]
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

    /// Returns a [`FallibleLender`] over [`RegisterEntry`] rows matching the
    /// query.
    ///
    /// The iterator owns the running cumulative [`Amount`], and each lent
    /// `RegisterEntry` borrows from it. This makes the per-row cost a few
    /// reference fields rather than an `Amount` clone, which matters for UI
    /// consumers that just want to display the rows as they stream by.
    ///
    /// Per-row conversion may fail (missing rate at the transaction date), so
    /// iteration is fallible via [`FallibleLender`] and yields
    /// [`QueryError`] on error.
    ///
    /// Only [`ConversionStrategy::Historical`] is supported when conversion is
    /// requested; [`ConversionStrategy::UpToDate`] returns
    /// [`QueryError::UnsupportedConversionStrategy`]. See #313.
    pub fn register_entries<'a>(
        &'a mut self,
        ctx: &'a ReportContext<'ctx>,
        query: &RegisterQuery<'ctx>,
    ) -> Result<RegisterEntries<'a, 'ctx>, QueryError> {
        let conversion = match query.conversion {
            None => None,
            Some(c) => match c.strategy {
                ConversionStrategy::Historical => Some(c),
                ConversionStrategy::UpToDate { .. } => {
                    return Err(QueryError::UnsupportedConversionStrategy);
                }
            },
        };
        // When the account filter excludes every known account, return an
        // iterator that is already exhausted instead of scanning everything.
        let account_filter = AccountFilter::new(ctx, query.account.as_deref());
        let txns: std::slice::Iter<'a, Transaction<'ctx>> = match account_filter {
            None => [].iter(),
            Some(_) => self.transactions.iter(),
        };
        Ok(RegisterEntries {
            ctx,
            txns,
            current: [].iter(),
            account_filter: account_filter.unwrap_or(AccountFilter::Any),
            date_range: query.date_range,
            conversion,
            price_repos: &mut self.price_repos,
            current_date: NaiveDate::MIN,
            current_amount: Amount::default(),
            total: Amount::default(),
        })
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
                        .map_err(QueryError::CommodityConversionFailure)?,
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
                        .map_err(QueryError::CommodityConversionFailure)?,
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
            .map_err(QueryError::CommodityConversionFailure)?,
        };
        Ok(evaled)
    }
}

/// [`FallibleLender`] returned by [`Ledger::register_entries`].
///
/// Holds the running cumulative [`Amount`] and a borrow into the underlying
/// transactions; each call to [`FallibleLender::next`] advances to the next
/// matching posting, updates the running total, and lends a borrow. Errors
/// from currency conversion are surfaced as [`QueryError`].
pub struct RegisterEntries<'a, 'ctx> {
    ctx: &'a ReportContext<'ctx>,
    txns: std::slice::Iter<'a, Transaction<'ctx>>,
    current: std::slice::Iter<'a, Posting<'ctx>>,
    account_filter: AccountFilter<'ctx>,
    date_range: DateRange,
    conversion: Option<Conversion<'ctx>>,
    price_repos: &'a mut PriceRepository<'ctx>,
    current_date: NaiveDate,
    /// Buffer holding the amount lent in the most recent yield. Storing this
    /// inside the lender keeps `RegisterEntry::amount` a `&Amount` even when
    /// conversion produces a fresh value.
    current_amount: Amount<'ctx>,
    total: Amount<'ctx>,
}

impl<'a, 'ctx> RegisterEntries<'a, 'ctx> {
    fn advance_to_next_posting(&mut self) -> Option<&'a Posting<'ctx>> {
        loop {
            if let Some(posting) = self.current.next() {
                if self.account_filter.is_match(&posting.account) {
                    return Some(posting);
                }
                continue;
            }
            let txn = self.txns.next()?;
            if !self.date_range.contains(txn.date) {
                continue;
            }
            self.current_date = txn.date;
            self.current = txn.postings.iter();
        }
    }
}

impl<'a, 'ctx, 'lend> FallibleLending<'lend> for RegisterEntries<'a, 'ctx> {
    type Lend = RegisterEntry<'lend, 'ctx>;
}

impl<'a, 'ctx> FallibleLender for RegisterEntries<'a, 'ctx> {
    type Error = QueryError;

    check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<lender::FallibleLend<'_, Self>>, Self::Error> {
        let Some(posting) = self.advance_to_next_posting() else {
            return Ok(None);
        };
        self.current_amount = match self.conversion {
            None => posting.amount.clone(),
            Some(conv) => {
                // ConversionStrategy::UpToDate is rejected up-front by
                // register_entries(); only Historical reaches this point.
                debug_assert!(matches!(conv.strategy, ConversionStrategy::Historical));
                price_db::convert_amount(
                    self.ctx,
                    self.price_repos,
                    &posting.amount,
                    conv.target,
                    self.current_date,
                )
                .map_err(QueryError::CommodityConversionFailure)?
            }
        };
        self.total += self.current_amount.clone();
        Ok(Some(RegisterEntry {
            date: self.current_date,
            payee: posting.payee,
            account: posting.account,
            amount: &self.current_amount,
            total: &self.total,
        }))
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

    use assert_matches::assert_matches;
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

    /// Owned snapshot of a `RegisterEntry`, used in tests to compare against
    /// expected rows without juggling the lender's borrow.
    type Row<'ctx> = (NaiveDate, String, Account<'ctx>, Amount<'ctx>, Amount<'ctx>);

    /// Drains the lender into an owned vec for easy assertion. The `total`
    /// borrow is materialized as an owned `Amount` snapshot per row.
    fn collect_register<'ctx>(
        mut entries: RegisterEntries<'_, 'ctx>,
    ) -> Result<Vec<Row<'ctx>>, QueryError> {
        let mut out = Vec::new();
        while let Some(entry) = entries.next()? {
            out.push((
                entry.date,
                entry.payee.to_owned(),
                entry.account,
                entry.amount.clone(),
                entry.total.clone(),
            ));
        }
        Ok(out)
    }

    #[test]
    fn register_entries_filter_by_account() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);
        let jpy = ctx.commodities.resolve("JPY").unwrap();

        let bank = ctx.account("Assets:J 銀行").unwrap();
        let got = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: Some("Assets:J 銀行".to_string()),
                        date_range: DateRange::default(),
                        conversion: None,
                    },
                )
                .unwrap(),
        )
        .unwrap();

        let want = vec![
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                "Initial".to_string(),
                bank,
                Amount::from_value(jpy, dec!(1000000)),
                Amount::from_value(jpy, dec!(1000000)),
            ),
            (
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                "Shopping".to_string(),
                bank,
                Amount::from_value(jpy, dec!(-17150)),
                Amount::from_value(jpy, dec!(982850)),
            ),
            (
                NaiveDate::from_ymd_opt(2024, 1, 9).unwrap(),
                "Buy Stock".to_string(),
                bank,
                Amount::from_value(jpy, dec!(-2760)),
                Amount::from_value(jpy, dec!(980090)),
            ),
        ];
        assert_eq!(want, got);
    }

    #[test]
    fn register_entries_filter_by_date_range() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let chf = ctx.commodities.resolve("CHF").unwrap();

        let grocery = ctx.account("Expenses:Grocery").unwrap();
        let bank = ctx.account("Assets:J 銀行").unwrap();
        let got = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: None,
                        date_range: DateRange {
                            start: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
                            end: Some(NaiveDate::from_ymd_opt(2024, 1, 9).unwrap()),
                        },
                        conversion: None,
                    },
                )
                .unwrap(),
        )
        .unwrap();

        // Only the 2024/01/05 Shopping transaction is in range; the 2024/01/09
        // Buy Stock is excluded by the half-open end. Running total is the
        // cumulative sum of the displayed amounts, mixing commodities.
        let want = vec![
            (
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                "Shopping".to_string(),
                grocery,
                Amount::from_value(chf, dec!(100.00)),
                Amount::from_value(chf, dec!(100.00)),
            ),
            (
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                "Shopping".to_string(),
                bank,
                Amount::from_value(jpy, dec!(-17150)),
                Amount::from_iter([(chf, dec!(100.00)), (jpy, dec!(-17150))]),
            ),
        ];
        assert_eq!(want, got);
    }

    #[test]
    fn register_entries_unknown_account_is_empty() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

        let got = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: Some("Does:Not:Exist".to_string()),
                        date_range: DateRange::default(),
                        conversion: None,
                    },
                )
                .unwrap(),
        )
        .unwrap();
        assert!(got.is_empty(), "expected empty register, got {:?}", got);
    }

    #[test]
    fn register_entries_historical_conversion() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let grocery = ctx.account("Expenses:Grocery").unwrap();
        let bank = ctx.account("Assets:J 銀行").unwrap();

        // Restrict to the single 2024/01/05 Shopping transaction so the
        // assertion stays small. Conversion is to JPY at the historical rate
        // captured by the @ price (171.50 JPY per CHF on 2024-01-05).
        let got = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: None,
                        date_range: DateRange {
                            start: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
                            end: Some(NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()),
                        },
                        conversion: Some(Conversion {
                            strategy: ConversionStrategy::Historical,
                            target: jpy,
                        }),
                    },
                )
                .unwrap(),
        )
        .unwrap();

        let want = vec![
            (
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                "Shopping".to_string(),
                grocery,
                // 100.00 CHF * 171.50 JPY/CHF.
                Amount::from_value(jpy, dec!(17150.00)),
                Amount::from_value(jpy, dec!(17150.00)),
            ),
            (
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                "Shopping".to_string(),
                bank,
                Amount::from_value(jpy, dec!(-17150)),
                Amount::from_value(jpy, dec!(0.00)),
            ),
        ];
        assert_eq!(want, got);
    }

    #[test]
    fn register_entries_rejects_up_to_date_conversion() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);
        let jpy = ctx.commodities.resolve("JPY").unwrap();

        let err = ledger
            .register_entries(
                &ctx,
                &RegisterQuery {
                    account: None,
                    date_range: DateRange::default(),
                    conversion: Some(Conversion {
                        strategy: ConversionStrategy::UpToDate {
                            today: NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
                        },
                        target: jpy,
                    }),
                },
            )
            .err()
            .expect("UpToDate is not supported yet");
        assert_matches!(
            err,
            QueryError::UnsupportedConversionStrategy,
            "unexpected error: {:?}",
            err
        );
    }

    #[test]
    fn register_entries_running_total_accumulates_across_accounts() {
        // With no account filter we walk every posting; the cumulative total
        // across all yielded rows must equal the per-commodity sum of every
        // amount in the ledger. The Initial transaction is balanced via
        // Equity so the grand total ends up at absolute zero across
        // commodities, but with rounding artifacts it's safer to compare
        // against the explicit sum.
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);

        let entries = ledger
            .register_entries(&ctx, &RegisterQuery::default())
            .unwrap();
        let rows = collect_register(entries).unwrap();
        assert!(!rows.is_empty());

        let final_total = rows.last().map(|r| r.4.clone()).unwrap();
        let mut summed = Amount::default();
        for r in &rows {
            summed += r.3.clone();
        }
        assert_eq!(summed, final_total);
    }
}
