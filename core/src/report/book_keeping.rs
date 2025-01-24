//! Contains book keeping logics to process the input stream,
//! and convert them into a processed Transactions.

use std::{borrow::Borrow, path::PathBuf};

use bumpalo::collections as bcc;

use crate::{
    load,
    syntax::{
        self,
        decoration::AsUndecorated,
        tracked::{Tracked, TrackedSpan},
    },
};

use super::{
    balance::{Balance, BalanceError},
    context::ReportContext,
    error::{self, ReportError},
    eval::{Amount, EvalError, Evaluable, PostingAmount, SingleAmount},
    intern::InternError,
    query::Ledger,
    transaction::{Posting, Transaction},
};

/// Error related to transaction understanding.
// TODO: Reconsider the error in details.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BookKeepError {
    #[error("failed to evaluate the expression: {0}")]
    EvalFailure(#[from] EvalError),
    #[error("failed to meet balance condition: {0}")]
    BalanceFailure(#[from] BalanceError),
    #[error("posting amount must be resolved as a simple value with commodity or zero")]
    ComplexPostingAmount,
    #[error("transaction cannot have multiple postings without constraints")]
    UndeduciblePostingAmount(Tracked<usize>, Tracked<usize>),
    #[error("transaction cannot have unbalanced postings: {0}")]
    UnbalancedPostings(String),
    #[error("balance assertion failed: got {0} but expected {1}")]
    BalanceAssertionFailure(String, String),
    #[error("failed to register account: {0}")]
    InvalidAccount(#[source] InternError),
    #[error("failed to register commodity: {0}")]
    InvalidCommodity(#[source] InternError),
    #[error("posting without commodity should not have exchange")]
    ZeroAmountWithExchange(TrackedSpan),
    #[error("cost or lot exchange must not be zero")]
    ZeroExchangeRate(TrackedSpan),
    #[error("cost or lot exchange must have different commodity from the amount commodity")]
    ExchangeWithAmountCommodity {
        posting_amount: TrackedSpan,
        exchange: TrackedSpan,
    },
}

/// Options to control process behavior.
#[derive(Debug, Default)]
pub struct ProcessOptions {
    /// Path to the price DB file.
    pub price_db_path: Option<PathBuf>,
    /// Commodity used for conversion.
    pub conversion: Option<String>,
}

/// Takes the loader, and gives back the all read transactions.
/// Also returns the computed balance, as a side-artifact.
/// Usually this needs to be reordered, so just returning a `Vec`.
pub fn process<'ctx, L, F>(
    ctx: &mut ReportContext<'ctx>,
    loader: L,
    _options: &ProcessOptions,
) -> Result<Ledger<'ctx>, ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    let mut accum = ProcessAccumulator::new();
    loader.borrow().load(|path, pctx, entry| {
        accum.process(ctx, entry).map_err(|berr| {
            ReportError::BookKeep(
                berr,
                error::ErrorContext::new(
                    loader.borrow().error_style().clone(),
                    path.to_owned(),
                    pctx,
                ),
            )
        })
    })?;
    Ok(Ledger {
        transactions: accum.txns,
        raw_balance: accum.balance,
    })
}

struct ProcessAccumulator<'ctx> {
    balance: Balance<'ctx>,
    txns: Vec<Transaction<'ctx>>,
}

impl<'ctx> ProcessAccumulator<'ctx> {
    fn new() -> Self {
        Self {
            balance: Balance::default(),
            txns: Vec::new(),
        }
    }

    fn process(
        &mut self,
        ctx: &mut ReportContext<'ctx>,
        entry: &syntax::tracked::LedgerEntry,
    ) -> Result<(), BookKeepError> {
        match entry {
            syntax::LedgerEntry::Txn(txn) => {
                self.txns
                    .push(add_transaction(ctx, &mut self.balance, txn)?);
                Ok(())
            }
            syntax::LedgerEntry::Account(account) => {
                let canonical = ctx
                    .accounts
                    .insert_canonical(&account.name)
                    .map_err(BookKeepError::InvalidAccount)?;
                for ad in &account.details {
                    if let syntax::AccountDetail::Alias(alias) = ad {
                        ctx.accounts
                            .insert_alias(alias, canonical)
                            .map_err(BookKeepError::InvalidAccount)?;
                    }
                }
                Ok(())
            }
            syntax::LedgerEntry::Commodity(commodity) => {
                let canonical = ctx
                    .commodities
                    .insert_canonical(&commodity.name)
                    .map_err(BookKeepError::InvalidCommodity)?;
                for cd in &commodity.details {
                    match cd {
                        syntax::CommodityDetail::Alias(alias) => {
                            ctx.commodities
                                .insert_alias(alias, canonical)
                                .map_err(BookKeepError::InvalidCommodity)?;
                        }
                        syntax::CommodityDetail::Format(format_amount) => {
                            ctx.commodities
                                .set_format(canonical, format_amount.value.clone());
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
/// Adds a syntax transaction, and converts it into a processed Transaction.
fn add_transaction<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    bal: &mut Balance<'ctx>,
    txn: &syntax::tracked::Transaction,
) -> Result<Transaction<'ctx>, BookKeepError> {
    // First, process all postings, except the one without balance and amount,
    // which must be deduced later. And that should appear at most once.
    let mut postings = bcc::Vec::with_capacity_in(txn.posts.len(), ctx.arena);
    let mut unfilled: Option<Tracked<usize>> = None;
    let mut balance = Amount::default();
    for (i, posting) in txn.posts.iter().enumerate() {
        let posting_span = posting.span();
        let posting = posting.as_undecorated();
        let account = ctx.accounts.ensure(&posting.account);
        // amount stored in the posting, and balance delta.
        let (posting_value, balance_delta): (PostingAmount, PostingAmount) =
            match (&posting.amount, &posting.balance) {
                // posting with just `Account`, we need to deduce from other postings.
                (None, None) => {
                    if let Some(first) = unfilled.replace(Tracked::new(i, posting_span.clone())) {
                        Err(BookKeepError::UndeduciblePostingAmount(
                            first,
                            Tracked::new(i, posting_span.clone()),
                        ))
                    } else {
                        // placeholder which will be replaced later.
                        // balance_delta is also zero as we don't know the value yet.
                        Ok((PostingAmount::zero(), PostingAmount::zero()))
                    }
                }
                // posting with `Account   = X`.
                (None, Some(balance_constraints)) => {
                    let current: PostingAmount =
                        balance_constraints.as_undecorated().eval(ctx)?.try_into()?;
                    let prev: PostingAmount = bal.set_partial(account, current)?;
                    let amount = current.check_sub(prev)?;
                    Ok((amount, amount))
                }
                // regular posting with `Account    X`, optionally with balance.
                (Some(syntax_amount), balance_constraints) => {
                    let computed = compute_posting_amount(ctx, syntax_amount)?;
                    let expected_balance: Option<PostingAmount> = balance_constraints
                        .as_ref()
                        .map(|x| x.as_undecorated().eval(ctx))
                        .transpose()?
                        .map(|x| x.try_into())
                        .transpose()?;
                    let current = bal.add_posting_amount(account, computed.amount);
                    if let Some(expected) = expected_balance {
                        if !current.is_consistent(&expected) {
                            return Err(BookKeepError::BalanceAssertionFailure(
                                format!("{}", current.as_inline_display()),
                                format!("{}", expected),
                            ));
                        }
                    }
                    let balance_amount = calculate_balance_amount(syntax_amount, &computed)?;
                    Ok((computed.amount, balance_amount))
                }
            }?;
        balance += balance_delta;
        postings.push(Posting {
            account,
            amount: posting_value.into(),
        });
    }
    if let Some(u) = unfilled {
        let u = *u.as_undecorated();
        // Note that deduced amount can be multi-commodity, neither SingleAmount nor PostingAmount.
        let deduced: Amount = balance.clone().negate();
        postings[u].amount = deduced.clone();
        bal.add_amount(postings[u].account, deduced);
    } else {
        check_balance(ctx, balance)?;
    }
    Ok(Transaction {
        date: txn.date,
        postings: postings.into_boxed_slice(),
    })
}

/// Pre-copmuted syntax::Exchange.
enum Exchange<'ctx> {
    Total(SingleAmount<'ctx>),
    Rate(SingleAmount<'ctx>),
}

impl<'ctx> Exchange<'ctx> {
    fn is_zero(&self) -> bool {
        match self {
            Exchange::Total(x) => x.value.is_zero(),
            Exchange::Rate(x) => x.value.is_zero(),
        }
    }

    fn try_from_syntax<'a>(
        ctx: &mut ReportContext<'ctx>,
        syntax_amount: &syntax::tracked::PostingAmount<'a>,
        posting_amount: &PostingAmount<'ctx>,
        exchange: &syntax::tracked::Tracked<syntax::Exchange<'a>>,
    ) -> Result<Exchange<'ctx>, BookKeepError> {
        let (rate_commodity, rate) = match exchange.as_undecorated() {
            syntax::Exchange::Rate(rate) => {
                let rate: SingleAmount<'ctx> = rate.eval(ctx)?.try_into()?;
                (rate.commodity, Exchange::Rate(rate))
            }
            syntax::Exchange::Total(rate) => {
                let rate: SingleAmount<'ctx> = rate.eval(ctx)?.try_into()?;
                (rate.commodity, Exchange::Total(rate))
            }
        };
        if rate.is_zero() {
            return Err(BookKeepError::ZeroExchangeRate(exchange.span()));
        }
        match posting_amount {
            PostingAmount::Zero => Err(BookKeepError::ZeroAmountWithExchange(exchange.span())),
            PostingAmount::Single(amount) => {
                if amount.commodity == rate_commodity {
                    Err(BookKeepError::ExchangeWithAmountCommodity {
                        posting_amount: syntax_amount.amount.span(),
                        exchange: exchange.span(),
                    })
                } else {
                    Ok(rate)
                }
            }
        }
    }
}

struct ComputedPosting<'ctx> {
    amount: PostingAmount<'ctx>,
    cost: Option<Exchange<'ctx>>,
    lot: Option<Exchange<'ctx>>,
}

fn compute_posting_amount<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    syntax_amount: &syntax::tracked::PostingAmount<'_>,
) -> Result<ComputedPosting<'ctx>, BookKeepError> {
    let amount: PostingAmount = syntax_amount
        .amount
        .as_undecorated()
        .eval(ctx)?
        .try_into()?;
    let cost = posting_cost_exchange(syntax_amount)
        .map(|exchange| Exchange::try_from_syntax(ctx, syntax_amount, &amount, exchange))
        .transpose()?;
    let lot = posting_lot_exchange(syntax_amount)
        .map(|exchange| Exchange::try_from_syntax(ctx, syntax_amount, &amount, exchange))
        .transpose()?;
    Ok(ComputedPosting { amount, cost, lot })
}

/// Checks if the posting amounts sum to zero.
fn check_balance<'ctx>(
    ctx: &ReportContext<'ctx>,
    mut balance: Amount<'ctx>,
) -> Result<(), BookKeepError> {
    balance.round(|commodity| ctx.commodities.get_decimal_point(commodity));
    if balance.is_zero() {
        return Ok(());
    }
    if let Some((a1, a2)) = balance.maybe_pair() {
        log::debug!("deduced price {} == {}", a1, a2);
        return Ok(());
    }
    if !balance.is_zero() {
        return Err(BookKeepError::UnbalancedPostings(format!(
            "{}",
            balance.as_inline_display()
        )));
    }
    Ok(())
}

/// Returns the amount which sums up to the balance within the transaction.
fn calculate_balance_amount<'ctx>(
    _posting_amount: &syntax::tracked::PostingAmount,
    computed: &ComputedPosting<'ctx>,
) -> Result<PostingAmount<'ctx>, BookKeepError> {
    // Actually, there's no point to compute cost if lot price is provided.
    // Example: if you sell a X with cost p Y lot q Y.
    //   Broker          -a X {{q Y}} @@ p Y
    //   Broker           p Y
    //   Income   (-(p - q) Y)
    //
    // if you set the first posting amount t,
    // t + p Y - (p - q) Y = 0
    // t = -q Y
    // so actually cost is pointless in this case.
    match computed.lot.as_ref().or(computed.cost.as_ref()) {
        Some(x) => {
            let exchanged = calculate_exchanged_amount(x, computed.amount.try_into()?)?;
            Ok(exchanged.into())
        }
        None => Ok(computed.amount),
    }
}

/// Returns cost Exchange of the posting.
#[inline]
fn posting_cost_exchange<'a, 'ctx>(
    posting_amount: &'a syntax::tracked::PostingAmount<'ctx>,
) -> Option<&'a syntax::tracked::Tracked<syntax::Exchange<'ctx>>> {
    posting_amount.cost.as_ref()
}

/// Returns lot exchange of the posting.
#[inline]
fn posting_lot_exchange<'a, 'ctx>(
    posting_amount: &'a syntax::tracked::PostingAmount<'ctx>,
) -> Option<&'a syntax::tracked::Tracked<syntax::Exchange<'ctx>>> {
    posting_amount.lot.price.as_ref()
}

/// Given the exchange rate and the amount, returns the converted amount.
/// Note we don't need [`PriceRepository`][super::price_db::PriceRepository],
/// as we only tries to convert the posting with cost / lot information.
fn calculate_exchanged_amount<'ctx>(
    cost: &Exchange<'ctx>,
    amount: SingleAmount<'ctx>,
) -> Result<SingleAmount<'ctx>, BookKeepError> {
    let exchanged: Result<SingleAmount, EvalError> = match cost {
        Exchange::Rate(rate) => Ok(*rate * amount.value),
        Exchange::Total(abs) => Ok(abs.with_sign_of(amount)),
    };
    Ok(exchanged?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use chrono::NaiveDate;
    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::{
        parse::{self, testing::expect_parse_ok},
        syntax::tracked::TrackedSpan,
    };

    fn parse_transaction(input: &str) -> syntax::tracked::Transaction {
        let (_, ret) = expect_parse_ok(parse::transaction::transaction, input);
        ret
    }

    #[test]
    fn add_transaction_maintains_balance() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 1"),
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 1"),
            PostingAmount::from_value(dec!(123), ctx.commodities.ensure("EUR")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 2"),
            PostingAmount::from_value(dec!(1), ctx.commodities.ensure("EUR")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 4"),
            PostingAmount::from_value(dec!(10), ctx.commodities.ensure("CHF")),
        );
        let input = indoc! {"
            2024/08/01 Sample
              Account 1      200 JPY = 1200 JPY
              Account 2        0 JPY = 0 JPY
              Account 2     -100 JPY = -100 JPY
              Account 2     -100 JPY = -200 JPY
              Account 3     2.00 CHF @ 150 JPY
              Account 4              = -300 JPY
        "};
        let txn = parse_transaction(input);

        let _ = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");

        let want_balance: Balance = hashmap! {
            ctx.accounts.ensure("Account 1") =>
                Amount::from_values([
                    (dec!(1200), ctx.commodities.ensure("JPY")),
                    (dec!(123), ctx.commodities.ensure("EUR")),
                ]),
            ctx.accounts.ensure("Account 2") =>
                Amount::from_values([
                    (dec!(-200), ctx.commodities.ensure("JPY")),
                    (dec!(1), ctx.commodities.ensure("EUR")),
                ]),
            ctx.accounts.ensure("Account 3") =>
                Amount::from_value(dec!(2), ctx.commodities.ensure("CHF")),
            ctx.accounts.ensure("Account 4") =>
                Amount::from_values([
                    (dec!(-300), ctx.commodities.ensure("JPY")),
                    (dec!(10), ctx.commodities.ensure("CHF")),
                ]),
        }
        .into_iter()
        .collect();
        assert_eq!(want_balance.into_vec(), bal.into_vec());
    }

    #[test]
    fn add_transaction_emits_transaction_with_postings() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Account 1      200 JPY = 200 JPY
              Account 2     -100 JPY = -100 JPY
              Account 2     -100 JPY = -200 JPY
        "};
        let txn = parse_transaction(input);

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");

        let want = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(200), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(-100), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(-100), ctx.commodities.ensure("JPY")),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_emits_transaction_with_deduce_and_balance_concern() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 1"),
            PostingAmount::from_value(dec!(1000), ctx.commodities.ensure("JPY")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 1"),
            PostingAmount::from_value(dec!(123), ctx.commodities.ensure("USD")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 2"),
            PostingAmount::from_value(dec!(-100), ctx.commodities.ensure("JPY")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 2"),
            PostingAmount::from_value(dec!(-30), ctx.commodities.ensure("USD")),
        );
        bal.add_posting_amount(
            ctx.accounts.ensure("Account 3"),
            PostingAmount::from_value(dec!(-150), ctx.commodities.ensure("JPY")),
        );
        let input = indoc! {"
            2024/08/01 Sample
              Account 1              = 1200 JPY
              Account 2              = 0 JPY
              Account 3              = 0
              Account 4
        "};
        let txn = parse_transaction(input);

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");

        let want = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(200), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(100), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 3"),
                        amount: Amount::from_value(dec!(150), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 4"),
                        amount: Amount::from_value(dec!(-450), ctx.commodities.ensure("JPY")),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_deduced_amount_contains_multi_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Account 1         1200 JPY
              Account 2         234 EUR
              Account 3         34.56 CHF
              Account 4
        "};
        let txn = parse_transaction(input);
        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");
        let want = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(1200), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(234), ctx.commodities.ensure("EUR")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 3"),
                        amount: Amount::from_value(dec!(34.56), ctx.commodities.ensure("CHF")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 4"),
                        amount: Amount::from_values([
                            (dec!(-1200), ctx.commodities.ensure("JPY")),
                            (dec!(-234), ctx.commodities.ensure("EUR")),
                            (dec!(-34.56), ctx.commodities.ensure("CHF")),
                        ]),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_fails_when_two_posting_does_not_have_amount() {
        let input = indoc! {"
            2024/08/01 Sample
              Account 1 ; no amount
              Account 2 ; no amount
        "};
        let txn = parse_transaction(input);
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect_err("must fail");

        assert_eq!(
            got,
            BookKeepError::UndeduciblePostingAmount(
                Tracked::new(0, TrackedSpan::new(20..42)),
                Tracked::new(1, TrackedSpan::new(44..66))
            )
        );
    }
    #[test]
    fn add_transaction_fails_when_posting_has_zero_cost() {
        let input = indoc! {"
            2024/08/01 Sample
              Account 1            1 AAPL @ 0 USD
              Account 2          100 USD
        "};
        let txn = parse_transaction(input);
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect_err("must fail");

        assert_eq!(
            got,
            BookKeepError::ZeroExchangeRate(TrackedSpan::new(48..55))
        );
    }

    #[test]
    fn add_transaction_balances_with_lot() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Account 1             12 OKANE {100 JPY}
              Account 2         -1,200 JPY
        "};
        let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
        let txn = parse_transaction(input);

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");

        let okane = ctx.commodities.resolve("OKANE").unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let want = Transaction {
            date,
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(12), okane),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(-1200), jpy),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_balances_with_price() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Account 1             12 OKANE @ (1 * 100 JPY)
              Account 2         -1,200 JPY
        "};
        let txn = parse_transaction(input);
        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");
        let want = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(12), ctx.commodities.ensure("OKANE")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(-1200), ctx.commodities.ensure("JPY")),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_balances_with_lot_and_price() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Account 1            -12 OKANE {100 JPY} @ 120 JPY
              Account 2          1,440 JPY
              Income              -240 JPY
        "};
        let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
        let txn = parse_transaction(input);

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");

        let okane = ctx.commodities.resolve("OKANE").unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let want = Transaction {
            date,
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(-12), okane),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(1440), jpy),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Income"),
                        amount: Amount::from_value(dec!(-240), jpy),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_deduces_price_info() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Account 1            -12 OKANE
              Account 2          1,000 JPY
              Account 3            440 JPY
        "};
        let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
        let txn = parse_transaction(input);

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");

        let okane = ctx.commodities.resolve("OKANE").unwrap();
        let jpy = ctx.commodities.resolve("JPY").unwrap();
        let want = Transaction {
            date,
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(-12), okane),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(1000), jpy),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 3"),
                        amount: Amount::from_value(dec!(440), jpy),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }

    #[test]
    fn add_transaction_balances_minor_diff() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let chf = ctx.commodities.insert_canonical("CHF").unwrap();
        ctx.commodities
            .set_format(chf, "20,000.00".parse().unwrap());
        let mut bal = Balance::default();
        let input = indoc! {"
            2024/08/01 Sample
              Expenses               300 EUR @ (1 / 1.0538 CHF)
              Liabilities        -284.68 CHF
        "};
        let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
        let txn = parse_transaction(input);

        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");
        let eur = ctx.commodities.resolve("EUR").unwrap();
        let want = Transaction {
            date,
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Expenses"),
                        amount: Amount::from_value(dec!(300), eur),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Liabilities"),
                        amount: Amount::from_value(dec!(-284.68), chf),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }
}
