//! Parser about ledger postings.

use std::cmp::min;

use winnow::{
    ascii::space0,
    combinator::{cond, delimited, fail, opt, peek, preceded, terminated, trace},
    error::StrContext,
    stream::{AsChar, Stream, StreamIsPartial},
    token::{literal, one_of, take, take_till},
    PResult, Parser,
};

use crate::syntax;
use crate::{
    parse::{
        character::{line_ending_or_semi, paren, till_line_ending_or_semi},
        combinator::{cond_else, has_peek},
        expr, metadata, primitive,
    },
    syntax::decoration::Decoration,
};

pub fn posting<'i, Input, Deco>(input: &mut Input) -> PResult<syntax::Posting<'i, Deco>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    Deco: Decoration,
{
    trace("posting::posting", move |input: &mut Input| {
        let clear_state = preceded(space0, metadata::clear_state).parse_next(input)?;
        let account = posting_account
            .context(StrContext::Label("account of the posting"))
            .parse_next(input)?;
        let shortcut_amount = has_peek(line_ending_or_semi).parse_next(input)?;
        if shortcut_amount {
            let metadata = metadata::block_metadata.parse_next(input)?;
            return Ok(syntax::Posting {
                clear_state,
                metadata,
                ..syntax::Posting::new(account)
            });
        }
        let amount = opt(terminated(posting_amount, space0))
            .context(StrContext::Label("amount of the posting"))
            .parse_next(input)?;
        let balance =
            opt(
                Deco::decorate_parser(delimited((one_of('='), space0), expr::value_expr, space0))
                    .context(StrContext::Label("balance of the posting")),
            )
            .parse_next(input)?;
        let metadata = metadata::block_metadata
            .context(StrContext::Label("metadata section of the posting"))
            .parse_next(input)?;
        Ok(syntax::Posting {
            clear_state,
            amount,
            balance,
            metadata,
            ..syntax::Posting::new(account)
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

fn posting_amount<'i, Input, Deco>(input: &mut Input) -> PResult<syntax::PostingAmount<'i, Deco>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>
        + std::clone::Clone,
    <Input as Stream>::Token: AsChar + Clone,
    Deco: Decoration,
{
    let amount = terminated(Deco::decorate_parser(expr::value_expr), space0).parse_next(input)?;
    let lot = lot(input)?;
    let is_at = has_peek(one_of('@')).parse_next(input)?;
    let is_double_at = has_peek(literal("@@")).parse_next(input)?;
    let cost = cond(
        is_at,
        trace(
            "posting::posting_amount@cost",
            Deco::decorate_parser(cond_else(is_double_at, total_cost, rate_cost)),
        ),
    )
    .parse_next(input)?;
    Ok(syntax::PostingAmount { amount, cost, lot })
}

fn lot<'i, Input, Deco>(input: &mut Input) -> PResult<syntax::Lot<'i, Deco>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    Deco: Decoration,
{
    space0.void().parse_next(input)?;
    let mut lot = syntax::Lot::default();
    loop {
        let open = peek(opt(one_of(['(', '[', '{']))).parse_next(input)?;
        // TODO: Consider if we can implement this with cut_err and permutation.
        match open {
            None => return Ok(lot),
            Some('{') if lot.price.is_none() => {
                let amount = Deco::decorate_parser(lot_amount).parse_next(input)?;
                lot.price = Some(amount);
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

fn lot_amount<'i, Input>(input: &mut Input) -> PResult<syntax::Exchange<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    let is_total = has_peek(literal("{{")).parse_next(input)?;
    if is_total {
        delimited(
            (literal("{{"), space0),
            expr::value_expr,
            (space0, literal("}}")),
        )
        .parse_next(input)
        .map(syntax::Exchange::Total)
    } else {
        delimited(
            (literal("{"), space0),
            expr::value_expr,
            (space0, literal("}")),
        )
        .parse_next(input)
        .map(syntax::Exchange::Rate)
    }
}

fn total_cost<'i, Input>(input: &mut Input) -> PResult<syntax::Exchange<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    preceded((literal("@@"), space0), expr::value_expr)
        .map(syntax::Exchange::Total)
        .parse_next(input)
}

fn rate_cost<'i, Input>(input: &mut Input) -> PResult<syntax::Exchange<'i>>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
{
    preceded((literal("@"), space0), expr::value_expr)
        .map(syntax::Exchange::Rate)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;
    use syntax::plain;
    use winnow::LocatingSlice;

    use crate::{
        parse::testing::expect_parse_ok,
        syntax::{
            plain::{Lot, Posting, PostingAmount},
            pretty_decimal::PrettyDecimal,
        },
    };

    #[test]
    fn posting_cost_parses_valid_input() {
        assert_eq!(
            expect_parse_ok(posting_amount, "100 EUR @ 1.2 CHF"),
            (
                "",
                syntax::plain::PostingAmount {
                    amount: syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                        value: PrettyDecimal::unformatted(dec!(100)),
                        commodity: "EUR".into()
                    }),
                    cost: Some(syntax::Exchange::Rate(syntax::expr::ValueExpr::Amount(
                        syntax::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(1.2)),
                            commodity: "CHF".into(),
                        }
                    ))),
                    lot: syntax::Lot::default(),
                }
            )
        );
        assert_eq!(
            expect_parse_ok(posting_amount, "1000 EUR @@ 1,020 CHF"),
            (
                "",
                PostingAmount {
                    amount: syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                        value: PrettyDecimal::plain(dec!(1000)),
                        commodity: "EUR".into()
                    }),
                    cost: Some(syntax::Exchange::Total(syntax::expr::ValueExpr::Amount(
                        syntax::expr::Amount {
                            value: PrettyDecimal::comma3dot(dec!(1020)),
                            commodity: "CHF".into(),
                        }
                    ))),
                    lot: syntax::Lot::default()
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
                Posting {
                    amount: Some(
                        syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(1)),
                            commodity: "USD".into(),
                        })
                        .into()
                    ),
                    metadata: vec![
                        syntax::Metadata::KeyValueTag {
                            key: "Payee".into(),
                            value: syntax::MetadataValue::Text("My Card".into()),
                        },
                        syntax::Metadata::KeyValueTag {
                            key: "Date".into(),
                            value: syntax::MetadataValue::Expr("[2022-3-4]".into()),
                        },
                        syntax::Metadata::Comment("My card took commission".into()),
                        syntax::Metadata::WordTags(vec!["financial".into(), "経済".into(),],),
                    ],
                    ..Posting::new("Expenses:Commissions")
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
                Posting {
                    clear_state: syntax::ClearState::Pending,
                    ..Posting::new("Expenses")
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
                Posting {
                    clear_state: syntax::ClearState::Cleared,
                    amount: Some(PostingAmount {
                        amount: syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(100)),
                            commodity: "JPY".into()
                        }),
                        cost: None,
                        lot: Lot::default()
                    }),
                    ..Posting::new("Expenses")
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
                        preceded(space0, lot::<_, plain::Ident>)
                            .parse_peek(LocatingSlice::new(input.as_str()))
                            .expect_err("should fail");
                    } else {
                        assert_eq!(
                            expect_parse_ok(preceded(space0, lot), input.as_str()),
                            (
                                "",
                                syntax::plain::Lot {
                                    price: Some(syntax::Exchange::Rate(
                                        syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
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
