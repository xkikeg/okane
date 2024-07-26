//! Parser about ledger postings.

use std::cmp::min;

use winnow::{
    ascii::{space0, space1},
    combinator::{cond, delimited, fail, opt, peek, preceded, terminated, trace},
    error::StrContext,
    stream::{AsChar, Stream, StreamIsPartial},
    token::{literal, one_of, take, take_till},
    PResult, Parser,
};

use crate::parse::{
    character::{line_ending_or_semi, paren, till_line_ending_or_semi},
    combinator::{cond_else, has_peek},
    expr, metadata, primitive,
};
use crate::repl;

pub fn posting<'i, Input>(input: &mut Input) -> PResult<repl::Posting<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    trace("posting::posting", move |input: &mut Input| {
        let clear_state = preceded(space1, metadata::clear_state).parse_next(input)?;
        let account = posting_account
            .context(StrContext::Label("account of the posting"))
            .parse_next(input)?;
        let shortcut_amount = has_peek(line_ending_or_semi).parse_next(input)?;
        if shortcut_amount {
            let metadata = metadata::block_metadata.parse_next(input)?;
            return Ok(repl::Posting {
                clear_state,
                metadata,
                ..repl::Posting::new(account)
            });
        }
        let amount = opt(terminated(posting_amount, space0))
            .context(StrContext::Label("amount of the posting"))
            .parse_next(input)?;
        let balance = opt(delimited((one_of('='), space0), expr::value_expr, space0))
            .context(StrContext::Label("balance of the posting"))
            .parse_next(input)?;
        let metadata = metadata::block_metadata
            .context(StrContext::Label("metadata section of the posting"))
            .parse_next(input)?;
        Ok(repl::Posting {
            clear_state,
            amount,
            balance,
            metadata,
            ..repl::Posting::new(account)
        })
    })
    .context(StrContext::Label("posting of the transaction"))
    .parse_next(input)
}

/// Parses the posting account name, and consumes the trailing spaces and tabs.
fn posting_account<'i, Input>(input: &mut Input) -> PResult<<Input as Stream>::Slice>
where
    Input: Stream<Slice = &'i str> + StreamIsPartial + Clone,
    <Input as Stream>::Token: AsChar,
{
    let (_, line) = till_line_ending_or_semi.parse_peek(input.clone())?;
    let space = line.find("  ");
    let tab = line.find('\t');
    let length = match (space, tab) {
        (Some(x), Some(y)) => min(x, y),
        (Some(x), None) => x,
        (None, Some(x)) => x,
        _ => line.len(),
    };
    // Note space may be zero for the case amount / balance is omitted.
    terminated(take(length), space0).parse_next(input)
}

fn posting_amount<'i, Input>(input: &mut Input) -> PResult<repl::PostingAmount<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + std::clone::Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    let amount = terminated(expr::value_expr, space0).parse_next(input)?;
    let lot = lot(input)?;
    let is_at = has_peek(one_of('@')).parse_next(input)?;
    let is_double_at = has_peek(literal("@@")).parse_next(input)?;
    let cost = cond(
        is_at,
        trace(
            "posting::posting_amount@cost",
            cond_else(is_double_at, total_cost, rate_cost),
        ),
    )
    .parse_next(input)?;
    Ok(repl::PostingAmount { amount, cost, lot })
}

fn lot<'i, Input>(input: &mut Input) -> PResult<repl::Lot<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    space0.void().parse_next(input)?;
    let mut lot = repl::Lot::default();
    loop {
        let open = peek(opt(one_of(['(', '[', '{']))).parse_next(input)?;
        // TODO: Consider if we can implement this with cut_err and permutation.
        match open {
            None => return Ok(lot),
            Some('{') if lot.price.is_none() => {
                let is_total = has_peek(literal("{{")).parse_next(input)?;
                if is_total {
                    let amount = delimited(
                        (literal("{{"), space0),
                        expr::value_expr,
                        (space0, literal("}}")),
                    )
                    .parse_next(input)?;
                    lot.price = Some(repl::Exchange::Total(amount));
                } else {
                    let amount = delimited(
                        (literal("{"), space0),
                        expr::value_expr,
                        (space0, literal("}")),
                    )
                    .parse_next(input)?;
                    lot.price = Some(repl::Exchange::Rate(amount));
                }
            }
            Some('{') => {
                return fail
                    .context(StrContext::Label("lot price duplicated"))
                    .parse_next(input)
            }
            Some('[') if lot.date.is_none() => {
                let date = delimited(
                    (one_of('['), space0),
                    primitive::date,
                    (space0, one_of(']')),
                )
                .parse_next(input)?;
                lot.date = Some(date);
            }
            Some('[') => {
                return fail
                    .context(StrContext::Label("lot date duplicated"))
                    .parse_next(input)
            }
            Some('(') if lot.note.is_none() => {
                let note = paren(take_till(1.., ['(', ')', '@'])).parse_next(input)?;
                lot.note = Some(note.into());
            }
            Some('(') => {
                return fail
                    .context(StrContext::Label("lot note duplicated"))
                    .parse_next(input)
            }
            Some(c) => unreachable!("unexpected lot opening {}", c),
        }
        space0.parse_next(input)?;
    }
}

fn total_cost<'i, Input>(input: &mut Input) -> PResult<repl::Exchange<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    preceded((literal("@@"), space0), expr::value_expr)
        .map(repl::Exchange::Total)
        .parse_next(input)
}

fn rate_cost<'i, Input>(input: &mut Input) -> PResult<repl::Exchange<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    preceded((literal("@"), space0), expr::value_expr)
        .map(repl::Exchange::Rate)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse::testing::expect_parse_ok, repl::pretty_decimal::PrettyDecimal};

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;
    use smallvec::smallvec;

    #[test]
    fn posting_cost_parses_valid_input() {
        assert_eq!(
            expect_parse_ok(posting_amount, "100 EUR @ 1.2 CHF"),
            (
                "",
                repl::PostingAmount {
                    amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                        value: PrettyDecimal::unformatted(dec!(100)),
                        commodity: "EUR".into()
                    }),
                    cost: Some(repl::Exchange::Rate(repl::expr::ValueExpr::Amount(
                        repl::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(1.2)),
                            commodity: "CHF".into(),
                        }
                    ))),
                    lot: repl::Lot::default(),
                }
            )
        );
        assert_eq!(
            expect_parse_ok(posting_amount, "1000 EUR @@ 1,020 CHF"),
            (
                "",
                repl::PostingAmount {
                    amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                        value: PrettyDecimal::plain(dec!(1000)),
                        commodity: "EUR".into()
                    }),
                    cost: Some(repl::Exchange::Total(repl::expr::ValueExpr::Amount(
                        repl::expr::Amount {
                            value: PrettyDecimal::comma3dot(dec!(1020)),
                            commodity: "CHF".into(),
                        }
                    ))),
                    lot: repl::Lot::default()
                }
            )
        );
    }

    #[test]
    fn posting_many_comments() {
        let input: &str = " Expenses:Commissions    1 USD ; Payee: My Card\n ;Date ::  [2022-3-4] \n ; My card took commission\n ; :financial:経済:\n";
        assert_eq!(
            expect_parse_ok(posting, input),
            (
                "",
                repl::Posting {
                    amount: Some(
                        repl::expr::ValueExpr::Amount(repl::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(1)),
                            commodity: "USD".into(),
                        })
                        .into()
                    ),
                    metadata: smallvec![
                        repl::Metadata::KeyValueTag {
                            key: "Payee".into(),
                            value: repl::MetadataValue::Text("My Card".into()),
                        },
                        repl::Metadata::KeyValueTag {
                            key: "Date".into(),
                            value: repl::MetadataValue::Expr("[2022-3-4]".into()),
                        },
                        repl::Metadata::Comment("My card took commission".into()),
                        repl::Metadata::WordTags(smallvec!["financial".into(), "経済".into(),],),
                    ],
                    ..repl::Posting::new("Expenses:Commissions")
                }
            )
        )
    }

    #[test]
    fn posting_clear_state_pending() {
        let input = " !Expenses\n";
        assert_eq!(
            expect_parse_ok(posting, input),
            (
                "",
                repl::Posting {
                    clear_state: repl::ClearState::Pending,
                    ..repl::Posting::new("Expenses")
                }
            )
        );
    }

    #[test]
    fn posting_clear_state_cleared() {
        let input = " *  Expenses   100 JPY\n";
        assert_eq!(
            expect_parse_ok(posting, input),
            (
                "",
                repl::Posting {
                    clear_state: repl::ClearState::Cleared,
                    amount: Some(repl::PostingAmount {
                        amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(100)),
                            commodity: "JPY".into()
                        }),
                        cost: None,
                        lot: repl::Lot::default()
                    }),
                    ..repl::Posting::new("Expenses")
                }
            )
        );
    }

    #[test]
    fn posting_account_returns_minimal() {
        let input = indoc! {"
            Account Value     ;
            Next Account Value
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            (";\nNext Account Value\n", "Account Value")
        );
        let input = indoc! {"
            Account Value\t\t
            Next Account Value
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            ("\nNext Account Value\n", "Account Value")
        );
        let input = indoc! {"
            Account Value
            Next Account Value
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            ("\nNext Account Value\n", "Account Value")
        );
    }

    #[test]
    fn lot_random_input() {
        let segment = [" {  200 JPY }", " [ 2022/09/01 ]", "  (note foobar)"];
        for i in 0..=2 {
            for j in 0..=2 {
                for k in 0..=2 {
                    let want_fail = i == j || j == k || i == k;
                    let input = format!("{}{}{}", segment[i], segment[j], segment[k]);
                    if want_fail {
                        preceded(space0, lot)
                            .parse_peek(input.as_str())
                            .expect_err("should fail");
                    } else {
                        assert_eq!(
                            expect_parse_ok(preceded(space0, lot), input.as_str()),
                            (
                                "",
                                repl::Lot {
                                    price: Some(repl::Exchange::Rate(
                                        repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                            value: PrettyDecimal::unformatted(dec!(200)),
                                            commodity: "JPY".into()
                                        })
                                    )),
                                    date: Some(NaiveDate::from_ymd_opt(2022, 9, 1).unwrap()),
                                    note: Some("note foobar".into()),
                                }
                            )
                        )
                    }
                }
            }
        }
    }
}
