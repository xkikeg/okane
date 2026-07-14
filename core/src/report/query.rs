//! Provides query of transactions / balances on the processed [Ledger] instance.

use std::{borrow::Cow, collections::HashSet};

use bumpalo::collections as bcc;
use chrono::NaiveDate;
use lender::{FallibleLender, FallibleLending, check_covariance_fallible};

use crate::{
    parse,
    report::{commodity::CommodityTag, eval::OwnedEvalError},
    syntax,
};

use super::{
    account::Account,
    balance::Balance,
    commodity::OwnedCommodity,
    context::ReportContext,
    eval::{Amount, EvalError, Evaluable},
    price_db::{self, ConversionError, PriceRepository},
    transaction::{Posting, Transaction},
};

/// Contains processed transactions, so that users can query information.
#[derive(Debug)]
pub struct Ledger<'ctx> {
    /// Arena from the report context — needed to allocate new `Posting`
    /// slice boxes when building the date-sorted clone cache.
    pub(super) arena: &'ctx bumpalo::Bump,
    pub(super) transactions: Vec<Transaction<'ctx>>,
    /// Lazily-computed clone of `transactions` sorted by date. `None` until
    /// the first query that needs date ordering.
    ///
    /// We store actual `Transaction` clones (not indices into `transactions`)
    /// so the date-ordered iteration is sequential — sequential reads are
    /// L1-cache-friendly, whereas the previous index-based design did a
    /// random-access lookup into `transactions[i]` per yielded row.
    pub(super) date_sorted_txns: Option<Vec<Transaction<'ctx>>>,
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

/// Specifies the iteration order of `register` rows.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    /// Preserve the order of appearance in the source file(s).
    #[default]
    Original,
    /// Stable sort by transaction date, ascending.
    Date,
}

/// Query to drive [`Ledger::register_entries`].
#[derive(Debug, Default)]
// TODO: non_exhaustive
pub struct RegisterQuery<'ctx> {
    /// Select the specified account if specified.
    /// Note this will be changed to a list of regex eventually.
    pub account: AccountFilter<'ctx>,
    /// Half-open date range to restrict transactions.
    pub date_range: DateRange,
    /// Optional currency conversion applied to every yielded amount and to the
    /// running total. Only [`ConversionStrategy::Historical`] is supported.
    /// See https://github.com/xkikeg/okane/issues/313.
    pub conversion: Option<Conversion<'ctx>>,
    /// Order in which the matching postings are yielded.
    pub sort: Sort,
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
    /// Select which accounts to include. Defaults to [`AccountFilter::All`]
    /// (every account).
    pub account: AccountFilter<'ctx>,
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

    /// Builds the date-sorted transaction clone cache on first call; a
    /// no-op on every subsequent call. Clones each `Transaction` (postings
    /// re-allocated in the same arena) into a new `Vec`, then stable-sorts
    /// the copy by date. The result is a sequentially-iterable view that
    /// preserves the original `transactions` Vec in file order.
    fn ensure_date_sorted_txns(&mut self) {
        if self.date_sorted_txns.is_some() {
            return;
        }
        let arena = self.arena;
        let mut sorted: Vec<Transaction<'ctx>> = self
            .transactions
            .iter()
            .map(|txn| {
                let mut postings = bcc::Vec::with_capacity_in(txn.postings.len(), arena);
                for p in txn.postings.iter() {
                    postings.push(p.clone());
                }
                Transaction {
                    date: txn.date,
                    postings: postings.into_boxed_slice(),
                }
            })
            .collect();
        sorted.sort_by_key(|t| t.date);
        self.date_sorted_txns = Some(sorted);
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
        let account_filter = query.account.clone();
        // When the account filter can never match, return an iterator that is
        // already exhausted instead of scanning everything.
        let txns: TxnIter<'a, 'ctx> = if account_filter.is_exhaustive_empty() {
            TxnIter::linear(&[])
        } else {
            match query.sort {
                Sort::Original => TxnIter::linear(&self.transactions),
                // Build (or reuse) the date-sorted clone cache. Bounds are
                // applied via `date_range_iter` (binary-search lo, incremental
                // `end` check), so the inner `date_range.contains` check below
                // is a no-op on every txn yielded here — it's kept for the
                // `Linear` arms above.
                Sort::Date => {
                    self.ensure_date_sorted_txns();
                    date_range_iter(
                        self.date_sorted_txns
                            .as_deref()
                            .expect("just built by ensure_date_sorted_txns"),
                        query.date_range,
                    )
                }
            }
        };
        Ok(RegisterEntries {
            ctx,
            txns,
            current: [].iter(),
            account_filter,
            date_range: query.date_range,
            conversion,
            price_repos: &mut self.price_repos,
            current_date: NaiveDate::MIN,
            current_amount: Cow::Owned(Amount::default()),
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
            // Accumulating into Balance is commutative, so we are free to
            // pick the iteration order. When the date range is bounded we
            // use the date-sorted permutation (binary-search lower bound,
            // incremental upper bound); otherwise we walk every transaction
            // in source order.
            let txns = if query.date_range.is_bypass() {
                TxnIter::linear(&self.transactions)
            } else {
                self.ensure_date_sorted_txns();
                date_range_iter(
                    self.date_sorted_txns
                        .as_deref()
                        .expect("just built by ensure_date_sorted_txns"),
                    query.date_range,
                )
            };
            Cow::Owned(compute_balance(
                ctx,
                &mut self.price_repos,
                txns,
                query.conversion,
            )?)
        };
        let balance = match query.conversion {
            None
            | Some(Conversion {
                strategy: ConversionStrategy::Historical,
                ..
            }) => balance,
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
                Cow::Owned(converted)
            }
        };
        // Restrict to the requested accounts. `Any` short-circuits so the
        // common unfiltered query keeps returning the borrowed/cached balance
        // without an extra copy.
        match &query.account {
            AccountFilter::All => Ok(balance),
            filter => {
                let mut filtered = Balance::default();
                for (account, amount) in balance.iter() {
                    if filter.is_match(account) {
                        filtered.add_amount(*account, amount.clone());
                    }
                }
                Ok(Cow::Owned(filtered))
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

/// Iterator over transactions used internally by [`RegisterEntries`] and
/// [`Ledger::balance`].
///
/// Both the linear (file-order) and sorted (date-order) paths walk a
/// `slice::Iter<Transaction>` — the only difference is whether to
/// short-circuit on an upper-bound date. So we collapse them into a single
/// struct rather than an enum: no tag dispatch, no payload-size waste from
/// the larger variant, and the `end == None` case compiles to a single
/// well-predicted branch on the hot path.
pub(super) struct TxnIter<'a, 'ctx> {
    it: std::slice::Iter<'a, Transaction<'ctx>>,
    /// Upper bound for sorted iteration: once a yielded transaction's date
    /// is `>= end`, iteration stops. `None` for the linear / unbounded case.
    end: Option<NaiveDate>,
}

impl<'a, 'ctx> TxnIter<'a, 'ctx> {
    /// Walk `slice` in source order. No early-exit.
    fn linear(slice: &'a [Transaction<'ctx>]) -> Self {
        Self {
            it: slice.iter(),
            end: None,
        }
    }

    /// Walk `slice` (assumed to be in ascending date order) and stop as
    /// soon as a transaction's date is `>= end`. Pass `end = None` to
    /// disable the upper-bound check.
    fn sorted(slice: &'a [Transaction<'ctx>], end: Option<NaiveDate>) -> Self {
        Self {
            it: slice.iter(),
            end,
        }
    }
}

impl<'a, 'ctx> Iterator for TxnIter<'a, 'ctx> {
    type Item = &'a Transaction<'ctx>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let txn = self.it.next()?;
        if let Some(end) = self.end
            && txn.date >= end
        {
            return None;
        }
        Some(txn)
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
    txns: TxnIter<'a, 'ctx>,
    current: std::slice::Iter<'a, Posting<'ctx>>,
    account_filter: AccountFilter<'ctx>,
    date_range: DateRange,
    conversion: Option<Conversion<'ctx>>,
    price_repos: &'a mut PriceRepository<'ctx>,
    current_date: NaiveDate,
    /// Buffer holding the amount lent in the most recent yield. Storing this
    /// inside the lender keeps `RegisterEntry::amount` a `&Amount` even when
    /// conversion produces a fresh value.
    current_amount: Cow<'a, Amount<'ctx>>,
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
            None => Cow::Borrowed(&posting.amount),
            Some(conv) => {
                // ConversionStrategy::UpToDate is rejected up-front by
                // register_entries(); only Historical reaches this point.
                debug_assert!(matches!(conv.strategy, ConversionStrategy::Historical));
                Cow::Owned(
                    price_db::convert_amount(
                        self.ctx,
                        self.price_repos,
                        &posting.amount,
                        conv.target,
                        self.current_date,
                    )
                    .map_err(QueryError::CommodityConversionFailure)?,
                )
            }
        };
        self.total += self.current_amount.as_ref();
        Ok(Some(RegisterEntry {
            date: self.current_date,
            payee: posting.payee,
            account: posting.account,
            amount: &self.current_amount,
            total: &self.total,
        }))
    }
}

/// Builds a sorted [`TxnIter`] over the transactions matching `date_range`,
/// in ascending date order. The lower bound is found by binary search; the
/// upper bound is enforced incrementally by [`TxnIter::next`].
///
/// Free function (rather than a `&self` method on `Ledger`) so its borrow
/// of the cache is disjoint from `&mut self.price_repos` at the call site —
/// through a method we'd hold a shared borrow on all of `self`.
fn date_range_iter<'a, 'ctx>(
    sorted_txns: &'a [Transaction<'ctx>],
    date_range: DateRange,
) -> TxnIter<'a, 'ctx> {
    let lo = match date_range.start {
        None => 0,
        Some(start) => sorted_txns.partition_point(|t| t.date < start),
    };
    TxnIter::sorted(&sorted_txns[lo..], date_range.end)
}

/// Accumulates the balance for every posting yielded by `txns`, applying
/// `conversion` if requested.
///
/// Free function so the caller can split borrows: `price_repos` and the
/// iterator borrow disjoint fields of `Ledger`, but the borrow checker
/// only sees that split through a free function signature.
fn compute_balance<'a, 'ctx>(
    ctx: &ReportContext<'ctx>,
    price_repos: &mut PriceRepository<'ctx>,
    txns: TxnIter<'a, 'ctx>,
    conversion: Option<Conversion<'ctx>>,
) -> Result<Balance<'ctx>, QueryError> {
    let mut bal = Balance::default();
    for txn in txns {
        for posting in txn.postings.iter() {
            let delta = match conversion {
                Some(Conversion {
                    strategy: ConversionStrategy::Historical,
                    target,
                }) => Cow::Owned(
                    price_db::convert_amount(ctx, price_repos, &posting.amount, target, txn.date)
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
    }
    bal.round(ctx);
    Ok(bal)
}

/// Describes which accounts to include in a query.
#[derive(Debug, Clone, Default)]
pub enum AccountFilter<'ctx> {
    /// Matches every account.
    #[default]
    All,
    /// Matches exactly one account (O(1) equality check).
    Exact(Account<'ctx>),
    /// Matches any account in the set.
    Set(HashSet<Account<'ctx>>),
}

impl<'ctx> AccountFilter<'ctx> {
    /// Returns a filter matching exactly one already-resolved account.
    pub fn single(account: Account<'ctx>) -> Self {
        Self::Exact(account)
    }

    /// Constructs a filter matching any of the given set of accounts.
    ///
    /// Note this may return the `Exact` variant for optimization.
    fn from_set(accounts: HashSet<Account<'ctx>>) -> Self {
        if accounts.len() == 1 {
            Self::Exact(accounts.into_iter().next().unwrap())
        } else {
            Self::Set(accounts)
        }
    }

    /// Constructs a filter matching any of the given account names verbatim.
    ///
    /// Names that don't resolve to a known account simply contribute nothing;
    /// if none of them resolve, the returned filter matches no account.
    pub fn from_exact_accounts(ctx: &ReportContext<'ctx>, accounts: &[impl AsRef<str>]) -> Self {
        let mut matched = HashSet::new();
        for name in accounts {
            if let Some(account) = ctx.account(name.as_ref()) {
                matched.insert(account);
            }
        }
        Self::from_set(matched)
    }

    /// Builds a filter matching every known account that matches any of the
    /// given regex `patterns`.
    ///
    /// Each entry of `patterns` is a regular expression matched (unanchored,
    /// like `ledger`) against every known account name. With an empty
    /// `patterns`, the returned filter matches no account (callers that want
    /// "match every account" should use [`AccountFilter::All`] directly).
    ///
    /// Returns [`regex::Error`] if any pattern fails to compile.
    pub fn from_regex_patterns(
        ctx: &ReportContext<'ctx>,
        patterns: &[impl AsRef<str>],
    ) -> Result<Self, regex::Error> {
        let mut matched: HashSet<Account<'ctx>> = HashSet::new();
        let set = regex::RegexSet::new(patterns.iter().map(AsRef::as_ref))?;
        for account in ctx.all_accounts_unsorted() {
            if set.is_match(account.as_str()) {
                matched.insert(account);
            }
        }
        Ok(Self::from_set(matched))
    }

    /// Returns `true` when the filter can never match any account.
    fn is_exhaustive_empty(&self) -> bool {
        matches!(self, AccountFilter::Set(s) if s.is_empty())
    }

    fn is_match(&self, account: &Account<'ctx>) -> bool {
        match self {
            AccountFilter::All => true,
            AccountFilter::Exact(target) => account == target,
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
    use maplit::{hashmap, hashset};
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
                    account: AccountFilter::All,
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
                    account: AccountFilter::All,
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
                    account: AccountFilter::All,
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
                        account: AccountFilter::single(bank),
                        date_range: DateRange::default(),
                        conversion: None,
                        sort: Sort::Original,
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
                        account: AccountFilter::All,
                        date_range: DateRange {
                            start: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
                            end: Some(NaiveDate::from_ymd_opt(2024, 1, 9).unwrap()),
                        },
                        conversion: None,
                        sort: Sort::Original,
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
                        account: AccountFilter::from_exact_accounts(&ctx, &["Does:Not:Exist"]),
                        date_range: DateRange::default(),
                        conversion: None,
                        sort: Sort::Original,
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
                        account: AccountFilter::All,
                        date_range: DateRange {
                            start: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
                            end: Some(NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()),
                        },
                        conversion: Some(Conversion {
                            strategy: ConversionStrategy::Historical,
                            target: jpy,
                        }),
                        sort: Sort::Original,
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
                    account: AccountFilter::All,
                    date_range: DateRange::default(),
                    conversion: Some(Conversion {
                        strategy: ConversionStrategy::UpToDate {
                            today: NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
                        },
                        target: jpy,
                    }),
                    sort: Sort::Original,
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
            summed += &r.3;
        }
        assert_eq!(summed, final_total);
    }

    /// Ledger fixture whose transactions are intentionally out of date order
    /// so we can tell `Sort::Original` apart from `Sort::Date`.
    fn unordered_loader() -> load::Loader<load::FakeFileSystem> {
        // Dates: 2024/03/01, 2024/01/01, 2024/02/01 — i.e. neither
        // ascending nor descending.
        let content = indoc! {"
            2024/03/01 Third by file
                Assets:Bank      300 JPY
                Equity

            2024/01/01 First by date
                Assets:Bank      100 JPY
                Equity

            2024/02/01 Middle by date
                Assets:Bank      200 JPY
                Equity
        "};
        let fake = hashmap! {
            PathBuf::from("path/to/file.ledger") => content.as_bytes().to_vec(),
        };
        load::Loader::new(PathBuf::from("path/to/file.ledger"), fake.into())
    }

    #[test]
    fn register_entries_sort_original_preserves_file_order() {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            unordered_loader(),
            &report::ProcessOptions::default(),
        )
        .unwrap();
        let bank = ctx.account("Assets:Bank").unwrap();
        let rows = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: AccountFilter::single(bank),
                        sort: Sort::Original,
                        ..Default::default()
                    },
                )
                .unwrap(),
        )
        .unwrap();
        let dates: Vec<NaiveDate> = rows.iter().map(|r| r.0).collect();
        assert_eq!(
            dates,
            vec![
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            ]
        );
        // Every row is on the same account, so the filter is exercised.
        assert!(rows.iter().all(|r| r.2 == bank));
    }

    #[test]
    fn register_entries_sort_date_yields_ascending_order() {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            unordered_loader(),
            &report::ProcessOptions::default(),
        )
        .unwrap();
        let bank = ctx.account("Assets:Bank").unwrap();
        let rows = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: AccountFilter::single(bank),
                        sort: Sort::Date,
                        ..Default::default()
                    },
                )
                .unwrap(),
        )
        .unwrap();
        let dates: Vec<NaiveDate> = rows.iter().map(|r| r.0).collect();
        assert_eq!(
            dates,
            vec![
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            ]
        );
    }

    #[test]
    fn register_entries_sort_date_running_total_matches_chronology() {
        // Running total must follow the visit order: 100 → 300 → 600.
        // (cf. Sort::Original would visit 300 → 400 → 600.)
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            unordered_loader(),
            &report::ProcessOptions::default(),
        )
        .unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let bank = ctx.account("Assets:Bank").unwrap();
        let rows = collect_register(
            ledger
                .register_entries(
                    &ctx,
                    &RegisterQuery {
                        account: AccountFilter::single(bank),
                        sort: Sort::Date,
                        ..Default::default()
                    },
                )
                .unwrap(),
        )
        .unwrap();
        let totals: Vec<Amount> = rows.into_iter().map(|r| r.4).collect();
        assert_eq!(
            totals,
            vec![
                Amount::from_value(jpy, dec!(100)),
                Amount::from_value(jpy, dec!(300)),
                Amount::from_value(jpy, dec!(600)),
            ]
        );
    }

    #[test]
    fn account_filter_from_regex_patterns_empty_matches_nothing() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        let filter = AccountFilter::from_regex_patterns(&ctx, &[] as &[&str]).unwrap();
        assert_matches!(filter, AccountFilter::Set(s) if s.is_empty());
    }

    #[test]
    fn account_filter_from_regex_patterns_matches_substring() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        // Unanchored regex: "Assets" matches every account starting with it.
        let filter = AccountFilter::from_regex_patterns(&ctx, &["Assets"]).unwrap();
        let matched = assert_matches!(filter, AccountFilter::Set(s) => s);
        assert_eq!(
            matched,
            HashSet::from([
                ctx.account("Assets:CH Bank").unwrap(),
                ctx.account("Assets:Broker").unwrap(),
                ctx.account("Assets:J 銀行").unwrap()
            ]),
        );
    }

    #[test]
    fn account_filter_from_regex_patterns_multiple_patterns_are_unioned() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        let filter = AccountFilter::from_regex_patterns(&ctx, &["^Income", "Card$"]).unwrap();
        assert_matches!(filter, AccountFilter::Set(s) => {
            assert_eq!(s, hashset![
                ctx.account("Income:Capital Gain").unwrap(),
                ctx.account("Liabilities:EUR Card").unwrap(),
                ]);
        });
    }

    #[test]
    fn account_filter_from_regex_patterns_single_match_collapses_to_exact() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        let filter = AccountFilter::from_regex_patterns(&ctx, &["Grocery"]).unwrap();
        let expected = ctx.account("Expenses:Grocery").unwrap();
        assert_matches!(filter, AccountFilter::Exact(a) if a == expected);
    }

    #[test]
    fn account_filter_from_exact_accounts_matches_names_verbatim() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        // "." would match everything if treated as regex; as an exact name it
        // resolves to nothing. The known exact account is added; the unknown
        // ones contribute nothing.
        let filter =
            AccountFilter::from_exact_accounts(&ctx, &["Assets:Broker", ".", "Does:Not:Exist"]);
        let expected = ctx.account("Assets:Broker").unwrap();
        assert_matches!(filter, AccountFilter::Exact(a) if a == expected);
    }

    #[test]
    fn account_filter_from_exact_accounts_no_matches_is_empty_set() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        let filter = AccountFilter::from_exact_accounts(&ctx, &["Does:Not:Exist"]);
        assert_matches!(filter, AccountFilter::Set(s) if s.is_empty());
    }

    #[test]
    fn account_filter_from_regex_patterns_invalid_regex_errors() {
        let arena = Bump::new();
        let (ctx, _ledger) = create_ledger(&arena);

        let err = AccountFilter::from_regex_patterns(&ctx, &["("]);
        assert!(err.is_err(), "expected regex compile error, got {err:?}");
    }

    #[test]
    fn balance_with_account_filter() {
        let arena = Bump::new();
        let (ctx, mut ledger) = create_ledger(&arena);
        let chf = ctx.commodities.resolve("CHF").unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let okane = ctx.commodities.resolve("OKANE").unwrap();

        let got = ledger
            .balance(
                &ctx,
                &BalanceQuery {
                    account: AccountFilter::from_regex_patterns(&ctx, &["^Assets"]).unwrap(),
                    conversion: None,
                    date_range: DateRange::default(),
                },
            )
            .unwrap()
            .into_owned();

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
        ]
        .into_iter()
        .collect();
        assert_eq!(want, got);
    }
}
