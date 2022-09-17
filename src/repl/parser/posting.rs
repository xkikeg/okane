//! Parser about ledger postings.

use crate::repl;
use repl::parser::{
    character,
    combinator::{cond_else, has_peek},
    expr, metadata,
};

use std::cmp::min;

use nom::{
    bytes::complete::{tag, take},
    character::complete::{char, space0, space1},
    combinator::{cond, opt, peek},
    error::{context, ParseError, VerboseError},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

pub fn posting(input: &str) -> IResult<&str, repl::Posting, VerboseError<&str>> {
    context("posting of the transaction", |input| {
        let (input, account) =
            context("account of the posting", preceded(space1, posting_account))(input)?;
        let (input, shortcut_amount) = has_peek(character::line_ending_or_semi)(input)?;
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
fn posting_account<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    let (input, line) = peek(character::not_line_ending_or_semi)(input)?;
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

fn posting_amount<'a>(input: &'a str) -> IResult<&str, repl::PostingAmount, VerboseError<&'a str>> {
    let (input, amount) = terminated(expr::value_expr, space0)(input)?;
    let (input, is_at) = has_peek(char('@'))(input)?;
    let (input, is_double_at) = has_peek(tag("@@"))(input)?;
    let (input, cost) = cond(
        is_at,
        context(
            "posting cost exchange",
            cond_else(is_double_at, total_cost, rate_cost),
        ),
    )(input)?;
    Ok((
        input,
        repl::PostingAmount {
            amount,
            cost,
            lot: repl::Lot::default(),
        },
    ))
}

fn total_cost<'a>(input: &'a str) -> IResult<&str, repl::Exchange, VerboseError<&'a str>> {
    let (input, v) = preceded(pair(tag("@@"), space0), expr::value_expr)(input)?;
    Ok((input, repl::Exchange::Total(v)))
}

fn rate_cost<'a>(input: &'a str) -> IResult<&str, repl::Exchange, VerboseError<&'a str>> {
    let (input, v) = preceded(pair(tag("@"), space0), expr::value_expr)(input)?;
    Ok((input, repl::Exchange::Rate(v)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

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
                    amount: repl::expr::ValueExpr::Amount(repl::Amount {
                        value: dec!(100),
                        commodity: "EUR".to_string()
                    }),
                    cost: Some(repl::Exchange::Rate(repl::expr::ValueExpr::Amount(
                        repl::Amount {
                            value: dec!(1.2),
                            commodity: "CHF".to_string(),
                        }
                    ))),
                    lot: repl::Lot::default(),
                }
            )
        );
        assert_eq!(
            expect_parse_ok(posting_amount, "100 EUR @@ 120 CHF"),
            (
                "",
                repl::PostingAmount {
                    amount: repl::expr::ValueExpr::Amount(repl::Amount {
                        value: dec!(100),
                        commodity: "EUR".to_string()
                    }),
                    cost: Some(repl::Exchange::Total(repl::expr::ValueExpr::Amount(
                        repl::Amount {
                            value: dec!(120),
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
        let input: &str = " Expenses:Commissions    1 USD ; Payee: My Card\n ; My card took commission\n ; :financial:経済:\n";
        assert_eq!(
            expect_parse_ok(posting, input),
            (
                "",
                repl::Posting {
                    amount: Some(
                        repl::expr::ValueExpr::Amount(repl::Amount {
                            value: dec!(1),
                            commodity: "USD".to_string(),
                        })
                        .into()
                    ),
                    metadata: vec![
                        repl::Metadata::KeyValueTag {
                            key: "Payee".to_string(),
                            value: "My Card".to_string(),
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
}
