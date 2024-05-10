//! Parser about ledger postings.

use crate::repl;
use repl::parser::{
    character::{line_ending_or_semi, not_line_ending_or_semi, paren},
    combinator::{cond_else, has_peek},
    expr, metadata, primitive,
};

use std::cmp::min;

use winnow::{
    bytes::complete::{is_not, tag, take},
    character::complete::{char, one_of, space0, space1},
    combinator::{cond, fail, map, opt, peek},
    error::{context, ParseError, VerboseError},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

pub fn posting(input: &str) -> IResult<&str, repl::Posting, VerboseError<&str>> {
    context("posting of the transaction", |input| {
        let (input, account) =
            context("account of the posting", preceded(space1, posting_account))(input)?;
        let (input, shortcut_amount) = has_peek(line_ending_or_semi)(input)?;
        if shortcut_amount {
            let (input, metadata) = metadata::block_metadata(input)?;
            return Ok((
                input,
                repl::Posting {
                    metadata,
                    ..repl::Posting::new(account.to_string())
                },
            ));
        }
        let (input, amount) = context(
            "amount of the posting",
            opt(terminated(posting_amount, space0)),
        )(input)?;
        let (input, balance) = context(
            "balance of the posting",
            opt(delimited(pair(char('='), space0), expr::value_expr, space0)),
        )(input)?;
        let (input, metadata) = metadata::block_metadata(input)?;
        Ok((
            input,
            repl::Posting {
                amount,
                balance,
                metadata,
                ..repl::Posting::new(account.to_string())
            },
        ))
    })(input)
}

/// Parses the posting account name, and consumes the trailing spaces and tabs.
fn posting_account<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let (input, line) = peek(not_line_ending_or_semi)(input)?;
    let space = line.find("  ");
    let tab = line.find('\t');
    let length = match (space, tab) {
        (Some(x), Some(y)) => min(x, y),
        (Some(x), None) => x,
        (None, Some(x)) => x,
        _ => line.len(),
    };
    // Note space may be zero for the case amount / balance is omitted.
    terminated(take(length), space0)(input)
}

fn posting_amount(input: &str) -> IResult<&str, repl::PostingAmount, VerboseError<&str>> {
    let (input, amount) = terminated(expr::value_expr, space0)(input)?;
    let (input, lot) = lot(input)?;
    let (input, is_at) = has_peek(char('@'))(input)?;
    let (input, is_double_at) = has_peek(tag("@@"))(input)?;
    let (input, cost) = cond(
        is_at,
        context(
            "posting cost exchange",
            cond_else(is_double_at, total_cost, rate_cost),
        ),
    )(input)?;
    Ok((input, repl::PostingAmount { amount, cost, lot }))
}

fn lot(input: &str) -> IResult<&str, repl::Lot, VerboseError<&str>> {
    let (mut input, _) = space0(input)?;
    let mut lot = repl::Lot::default();
    loop {
        let (_, open) = peek(opt(one_of("([{")))(input)?;
        match open {
            None => return Ok((input, lot)),
            Some('{') if lot.price.is_none() => {
                let (_, is_total) = has_peek(tag("{{"))(input)?;
                if is_total {
                    let (i1, amount) = delimited(
                        pair(tag("{{"), space0),
                        expr::value_expr,
                        pair(space0, tag("}}")),
                    )(input)?;
                    lot.price = Some(repl::Exchange::Total(amount));
                    input = i1;
                } else {
                    let (i1, amount) = delimited(
                        pair(tag("{"), space0),
                        expr::value_expr,
                        pair(space0, tag("}")),
                    )(input)?;
                    lot.price = Some(repl::Exchange::Rate(amount));
                    input = i1;
                }
            }
            Some('{') => return context("lot price duplicated", fail)(input),
            Some('[') if lot.date.is_none() => {
                let (i1, date) = delimited(
                    pair(char('['), space0),
                    primitive::date,
                    pair(space0, char(']')),
                )(input)?;
                lot.date = Some(date);
                input = i1;
            }
            Some('[') => return context("lot date duplicated", fail)(input),
            Some('(') if lot.note.is_none() => {
                let (i1, note) = paren(is_not("()@"))(input)?;
                lot.note = Some(note.to_string());
                input = i1;
            }
            Some('(') => return context("lot note duplicated", fail)(input),
            Some(c) => unreachable!("unexpected lot opening {}", c),
        }
        (input, _) = space0(input)?;
    }
}

fn total_cost(input: &str) -> IResult<&str, repl::Exchange, VerboseError<&str>> {
    map(
        preceded(pair(tag("@@"), space0), expr::value_expr),
        repl::Exchange::Total,
    )(input)
}

fn rate_cost(input: &str) -> IResult<&str, repl::Exchange, VerboseError<&str>> {
    map(
        preceded(pair(tag("@"), space0), expr::value_expr),
        repl::Exchange::Rate,
    )(input)
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
                        preceded(space0, lot)(&input).expect_err("should fail");
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
