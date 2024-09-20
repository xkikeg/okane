//! Contains book keeping logics to process the input stream,
//! and convert them into a processed Transactions.

use std::borrow::Borrow;

use bumpalo::collections as bcc;
use chrono::NaiveDate;

use crate::{
    load,
    syntax::{self, decoration::AsUndecorated},
};

use super::{
    balance::{Balance, BalanceError},
    context::{Account, ReportContext},
    error::{self, ReportError},
    eval::{Amount, EvalError, Evaluable, PostingAmount, SingleAmount},
    intern::InternError,
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
    #[error("transaction cannot have multiple postings without amount: {0} {1}")]
    UndeduciblePostingAmount(usize, usize),
    #[error("transaction cannot have unbalanced postings: {0}")]
    UnbalancedPostings(String),
    #[error("balance assertion failed: got {0} but expected {1}")]
    BalanceAssertionFailure(String, String),
    #[error("failed to register account: {0}")]
    InvalidAccount(#[source] InternError),
    #[error("failed to register commodity: {0}")]
    InvalidCommodity(#[source] InternError),
}

/// Takes the loader, and gives back the all read transactions.
/// Also returns the computed balance, as a side-artifact.
/// Usually this needs to be reordered, so just returning a `Vec`.
pub fn process<'ctx, L, F>(
    ctx: &mut ReportContext<'ctx>,
    loader: L,
) -> Result<(Vec<Transaction<'ctx>>, Balance<'ctx>), ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    let mut accum = ProcessAccumulator::new();
    loader.borrow().load_repl(|path, pctx, entry| {
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
    Ok((accum.txns, accum.balance))
}

struct ProcessAccumulator<'ctx> {
    balance: Balance<'ctx>,
    txns: Vec<Transaction<'ctx>>,
}

impl<'ctx> ProcessAccumulator<'ctx> {
    fn new() -> Self {
        let balance = Balance::default();
        let txns: Vec<Transaction<'ctx>> = Vec::new();
        Self { balance, txns }
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

/// Evaluated transaction, already processed to have right balance.
// TODO: Rename it to EvaluatedTxn?
#[derive(Debug, PartialEq, Eq)]
pub struct Transaction<'ctx> {
    pub date: NaiveDate,
    // Posting in the transaction.
    // Note this MUST be a Box instead of &[Posting],
    // as Posting is a [Drop] and we can't skip calling Drop,
    // otherwise we leave allocated memory for Amount HashMap.
    pub postings: bumpalo::boxed::Box<'ctx, [Posting<'ctx>]>,
}

/// Evaluated posting of the transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting<'ctx> {
    pub account: Account<'ctx>,
    /// Note this Amount is not PostingAmount,
    /// as deduced posting may have non-single commodity amount.
    pub amount: Amount<'ctx>,
}

/// Adds a repl transaction, and converts it into a processed Transaction.
fn add_transaction<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    bal: &mut Balance<'ctx>,
    txn: &syntax::tracked::Transaction,
) -> Result<Transaction<'ctx>, BookKeepError> {
    let mut postings = bcc::Vec::with_capacity_in(txn.posts.len(), ctx.arena);
    let mut unfilled: Option<usize> = None;
    let mut balance = Amount::default();
    for (i, posting) in txn.posts.iter().enumerate() {
        let account = ctx.accounts.ensure(&posting.as_undecorated().account);
        let (amount, balance_amount): (PostingAmount, PostingAmount) = match (
            &posting.as_undecorated().amount,
            &posting.as_undecorated().balance,
        ) {
            (None, None) => {
                if let Some(j) = unfilled.replace(i) {
                    Err(BookKeepError::UndeduciblePostingAmount(j, i))
                } else {
                    Ok((PostingAmount::zero(), PostingAmount::zero()))
                }
            }
            (None, Some(balance_constraints)) => {
                let current: PostingAmount =
                    balance_constraints.as_undecorated().eval(ctx)?.try_into()?;
                let prev: PostingAmount = bal.set_partial(account, current)?;
                let amount = current.check_sub(prev)?;
                Ok((amount, amount))
            }
            (Some(syntax_amount), balance_constraints) => {
                let amount: PostingAmount = syntax_amount
                    .amount
                    .as_undecorated()
                    .eval(ctx)?
                    .try_into()?;
                let expected_balance: Option<PostingAmount> = balance_constraints
                    .as_ref()
                    .map(|x| x.as_undecorated().eval(ctx))
                    .transpose()?
                    .map(|x| x.try_into())
                    .transpose()?;
                let current = bal.add_posting_amount(account, amount);
                if let Some(expected) = expected_balance {
                    if !current.is_consistent(expected) {
                        return Err(BookKeepError::BalanceAssertionFailure(
                            format!("{}", current.as_inline_display()),
                            format!("{}", expected),
                        ));
                    }
                }
                let balance_amount = calculate_balance_amount(ctx, syntax_amount, amount)?;
                Ok((amount, balance_amount))
            }
        }?;
        balance += balance_amount;
        postings.push(Posting {
            account,
            amount: amount.into(),
        });
    }
    if let Some(u) = unfilled {
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

fn check_balance<'ctx>(
    ctx: &ReportContext<'ctx>,
    mut balance: Amount<'ctx>,
) -> Result<(), BookKeepError> {
    balance.round(|commodity| ctx.commodities.get_decimal_point(commodity));
    if balance.is_zero() {
        return Ok(());
    }
    if let Some((a1, a2)) = balance.maybe_pair() {
        log::info!("deduced price {} == {}", a1, a2);
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

fn calculate_balance_amount<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    posting_amount: &syntax::tracked::PostingAmount,
    computed_amount: PostingAmount<'ctx>,
) -> Result<PostingAmount<'ctx>, BookKeepError> {
    let cost: Option<SingleAmount<'ctx>> = posting_amount
        .cost
        .as_ref()
        .map(|x| calculate_exchanged_amount(ctx, x.as_undecorated(), computed_amount.try_into()?))
        .transpose()?;
    let lot: Option<SingleAmount<'ctx>> = posting_amount
        .lot
        .price
        .as_ref()
        .map(|x| calculate_exchanged_amount(ctx, x.as_undecorated(), computed_amount.try_into()?))
        .transpose()?;
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
    Ok(lot.or(cost).map(Into::into).unwrap_or(computed_amount))
}

fn calculate_exchanged_amount<'ctx>(
    ctx: &mut ReportContext<'ctx>,
    cost: &syntax::Exchange,
    amount: SingleAmount<'ctx>,
) -> Result<SingleAmount<'ctx>, BookKeepError> {
    let exchanged: Result<SingleAmount, EvalError> = match cost {
        syntax::Exchange::Rate(x) => {
            let rate: SingleAmount = x.eval(ctx)?.try_into()?;
            Ok(rate * amount.value)
        }
        syntax::Exchange::Total(y) => {
            let abs: SingleAmount = y.eval(ctx)?.try_into()?;
            Ok(abs.with_sign_of(amount))
        }
    };
    Ok(exchanged?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::parse::{self, testing::expect_parse_ok};

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
            ctx.accounts.ensure("Account 4"),
            PostingAmount::from_value(dec!(10), ctx.commodities.ensure("CHF")),
        );
        let input = indoc! {"
            2024/08/01 Sample
              Account 1      200 JPY = 1200 JPY
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
                Amount::from_value(dec!(-200), ctx.commodities.ensure("JPY")),
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
        let input = indoc! {"
            2024/08/01 Sample
              Account 1              = 1200 JPY
              Account 2
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
                        amount: Amount::from_value(dec!(-200), ctx.commodities.ensure("JPY")),
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
        assert_eq!(got, BookKeepError::UndeduciblePostingAmount(0, 1));
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
        let txn = parse_transaction(input);
        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");
        let want = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Account 1"),
                        amount: Amount::from_value(dec!(-12), ctx.commodities.ensure("OKANE")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Account 2"),
                        amount: Amount::from_value(dec!(1440), ctx.commodities.ensure("JPY")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Income"),
                        amount: Amount::from_value(dec!(-240), ctx.commodities.ensure("JPY")),
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
        let txn = parse_transaction(input);
        let got = add_transaction(&mut ctx, &mut bal, &txn).expect("must succeed");
        let want = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            postings: bcc::Vec::from_iter_in(
                [
                    Posting {
                        account: ctx.accounts.ensure("Expenses"),
                        amount: Amount::from_value(dec!(300), ctx.commodities.ensure("EUR")),
                    },
                    Posting {
                        account: ctx.accounts.ensure("Liabilities"),
                        amount: Amount::from_value(dec!(-284.68), ctx.commodities.ensure("CHF")),
                    },
                ],
                &arena,
            )
            .into_boxed_slice(),
        };
        assert_eq!(want, got);
    }
}
