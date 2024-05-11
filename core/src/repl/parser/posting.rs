//! Parser about ledger postings.

use crate::repl;
use repl::parser::{
    character::{line_ending_or_semi, not_line_ending_or_semi, paren},
    combinator::{cond_else, has_peek},
    expr, metadata, primitive,
};

use std::cmp::min;

use winnow::{
    ascii::{space0, space1},
    combinator::{cond, delimited, fail, opt, peek, preceded, terminated, trace},
    error::StrContext,
    token::{one_of, tag, take, take_till1},
    PResult, Parser,
};

pub fn posting(input: &mut &str) -> PResult<repl::Posting> {
    trace("posting::posting", move |input: &mut &str| {
        let account = preceded(space1, posting_account)
            .context(StrContext::Label("account of the posting"))
            .parse_next(input)?;
        let shortcut_amount = has_peek(line_ending_or_semi).parse_next(input)?;
        if shortcut_amount {
            let metadata = metadata::block_metadata.parse_next(input)?;
            return Ok(repl::Posting {
                metadata,
                ..repl::Posting::new(account.to_string())
            });
        }
        let amount = opt(terminated(posting_amount, space0))
            .context(winnow::error::StrContext::Label("amount of the posting"))
            .parse_next(input)?;
        let balance = opt(delimited((one_of('='), space0), expr::value_expr, space0))
            .context(StrContext::Label("balance of the posting"))
            .parse_next(input)?;
        let metadata = metadata::block_metadata.parse_next(input)?;
        Ok(repl::Posting {
            amount,
            balance,
            metadata,
            ..repl::Posting::new(account.to_string())
        })
    })
    .context(StrContext::Label("posting of the transaction"))
    .parse_next(input)
}

/// Parses the posting account name, and consumes the trailing spaces and tabs.
fn posting_account<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let (_, line) = not_line_ending_or_semi.parse_peek(input)?;
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

fn posting_amount(input: &mut &str) -> PResult<repl::PostingAmount> {
    let amount = terminated(expr::value_expr, space0).parse_next(input)?;
    let lot = lot(input)?;
    let is_at = has_peek(one_of('@')).parse_next(input)?;
    let is_double_at = has_peek(tag("@@")).parse_next(input)?;
    let cost = cond(
        is_at,
        trace(
            "posting::posting_amount::cost",
            cond_else(is_double_at, total_cost, rate_cost),
        ),
    )
    .parse_next(input)?;
    Ok(repl::PostingAmount { amount, cost, lot })
}

fn lot(input: &mut &str) -> PResult<repl::Lot> {
    space0.void().parse_next(input)?;
    let mut lot = repl::Lot::default();
    loop {
        let open = peek(opt(one_of(['(', '[', '{']))).parse_next(input)?;
        // TODO Consider if we can implement this with cut_err and permutation.
        match open {
            None => return Ok(lot),
            Some('{') if lot.price.is_none() => {
                let is_total = has_peek(tag("{{")).parse_next(input)?;
                if is_total {
                    let amount =
                        delimited((tag("{{"), space0), expr::value_expr, (space0, tag("}}")))
                            .parse_next(input)?;
                    lot.price = Some(repl::Exchange::Total(amount));
                } else {
                    let amount =
                        delimited((tag("{"), space0), expr::value_expr, (space0, tag("}")))
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
                let note = paren(take_till1(['(', ')', '@'])).parse_next(input)?;
                lot.note = Some(note.to_string());
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

fn total_cost(input: &mut &str) -> PResult<repl::Exchange> {
    preceded((tag("@@"), space0), expr::value_expr)
        .map(repl::Exchange::Total)
        .parse_next(input)
}

fn rate_cost(input: &mut &str) -> PResult<repl::Exchange> {
    preceded((tag("@"), space0), expr::value_expr)
        .map(repl::Exchange::Rate)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::{parser::testing::expect_parse_ok, pretty_decimal::PrettyDecimal};

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn posting_cost_parses_valid_input() {
        assert_eq!(
            expect_parse_ok(posting_amount, "100 EUR @ 1.2 CHF"),
            (
                "",
                repl::PostingAmount {
                    amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                        value: PrettyDecimal::unformatted(dec!(100)),
                        commodity: "EUR".to_string()
                    }),
                    cost: Some(repl::Exchange::Rate(repl::expr::ValueExpr::Amount(
                        repl::expr::Amount {
                            value: PrettyDecimal::unformatted(dec!(1.2)),
                            commodity: "CHF".to_string(),
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
                        commodity: "EUR".to_string()
                    }),
                    cost: Some(repl::Exchange::Total(repl::expr::ValueExpr::Amount(
                        repl::expr::Amount {
                            value: PrettyDecimal::comma3dot(dec!(1020)),
                            commodity: "CHF".to_string(),
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
                            commodity: "USD".to_string(),
                        })
                        .into()
                    ),
                    metadata: vec![
                        repl::Metadata::KeyValueTag {
                            key: "Payee".to_string(),
                            value: repl::MetadataValue::Text("My Card".to_string()),
                        },
                        repl::Metadata::KeyValueTag {
                            key: "Date".to_string(),
                            value: repl::MetadataValue::Expr("[2022-3-4]".to_string()),
                        },
                        repl::Metadata::Comment("My card took commission".to_string()),
                        repl::Metadata::WordTags(
                            vec!["financial".to_string(), "経済".to_string(),],
                        ),
                    ],
                    ..repl::Posting::new("Expenses:Commissions".to_string())
                }
            )
        )
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
        let segment = vec![" {  200 JPY }", " [ 2022/09/01 ]", "  (note foobar)"];
        for i in 0..=2 {
            for j in 0..=2 {
                for k in 0..=2 {
                    let want_fail = i == j || j == k || i == k;
                    let input = format!("{}{}{}", segment[i], segment[j], segment[k]);
                    if want_fail {
                        preceded(space0, lot)
                            .parse_peek(&input)
                            .expect_err("should fail");
                    } else {
                        assert_eq!(
                            expect_parse_ok(preceded(space0, lot), &input),
                            (
                                "",
                                repl::Lot {
                                    price: Some(repl::Exchange::Rate(
                                        repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                            value: PrettyDecimal::unformatted(dec!(200)),
                                            commodity: "JPY".to_string()
                                        })
                                    )),
                                    date: Some(NaiveDate::from_ymd_opt(2022, 9, 1).unwrap()),
                                    note: Some("note foobar".to_string()),
                                }
                            )
                        )
                    }
                }
            }
        }
    }
}
