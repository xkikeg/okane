//! Defines parser functions for transaction.

use std::borrow::Cow;

use winnow::{
    ascii::{space0, space1},
    combinator::{cond, cut_err, opt, peek, preceded, repeat, terminated, trace},
    error::StrContext,
    token::one_of,
    PResult, Parser,
};

use crate::{
    parse::{character, combinator::has_peek, metadata, posting, primitive},
    repl,
};

/// Parses a transaction from given string.
pub fn transaction<'i>(input: &mut &'i str) -> PResult<repl::Transaction<'i>> {
    trace("transaction::transaction", move |input: &mut &'i str| {
        let date = trace(
            "transaction::transaction@date",
            primitive::date.context(StrContext::Label("transaction date")),
        )
        .parse_next(input)?;
        let effective_date = trace(
            "transaction::transaction@effective_date",
            opt(preceded(one_of('='), primitive::date)),
        )
        .parse_next(input)?;
        let is_shortest = has_peek(character::line_ending_or_eof).parse_next(input)?;
        // Date (and effective date) should be followed by space, unless followed by line_ending.
        cond(!is_shortest, space1).void().parse_next(input)?;
        let clear_state = metadata::clear_state(input)?;
        let code = opt(terminated(character::paren_str, space0)).parse_next(input)?;
        let payee =
            opt(character::till_line_ending_or_semi.map(str::trim_end)).parse_next(input)?;
        let metadata = metadata::block_metadata(input)?;
        let posts = repeat(0.., preceded(peek(one_of(' ')), cut_err(posting::posting)))
            .parse_next(input)?;
        Ok(repl::Transaction {
            effective_date,
            clear_state,
            code: code.map(Cow::Borrowed),
            posts,
            metadata,
            ..repl::Transaction::new(date, payee.unwrap_or(""))
        })
    })
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

    #[test]
    fn transaction_parses_valid_minimal() {
        let input = "2022/01/23\n";
        assert_eq!(
            expect_parse_ok(transaction, input),
            (
                "",
                repl::Transaction::new(NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(), "")
            )
        );
    }

    #[test]
    fn transaction_parses_valid_compact() {
        let input = indoc! {"
            2022/01/23=2022/01/28 *(code)Foo
             Expense A\t123,456.78 USD
             Liabilities B
        "};
        assert_eq!(
            expect_parse_ok(transaction, input),
            (
                "",
                repl::Transaction {
                    effective_date: Some(NaiveDate::from_ymd_opt(2022, 1, 28).unwrap()),
                    clear_state: repl::ClearState::Cleared,
                    code: Some(Cow::Borrowed("code")),
                    payee: Cow::Borrowed("Foo"),
                    posts: vec![
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                    value: PrettyDecimal::comma3dot(dec!(123456.78)),
                                    commodity: Cow::Borrowed("USD"),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            ..repl::Posting::new("Expense A")
                        },
                        repl::Posting::new("Liabilities B")
                    ],
                    ..repl::Transaction::new(NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(), "")
                }
            )
        );
    }

    #[test]
    fn transaction_parses_valid_complex() {
        let input = indoc! {"
            2022/01/23=2022/01/28 ! (code) Foo ; とりあえずのメモ
              ; :取引:
             Expense A\t\t-123,456.78 USD;  Note expense A
             ; Payee: Bar
             Liabilities B  12 JPY  =  -1000 CHF
             ; :tag1:他のタグ:
             Assets C    =0    ; Cのノート
             ; これなんだっけ
        "};
        assert_eq!(
            expect_parse_ok(transaction, input),
            (
                "",
                repl::Transaction {
                    effective_date: Some(NaiveDate::from_ymd_opt(2022, 1, 28).unwrap()),
                    clear_state: repl::ClearState::Pending,
                    code: Some(Cow::Borrowed("code")),
                    payee: Cow::Borrowed("Foo"),
                    metadata: vec![
                        repl::Metadata::Comment(Cow::Borrowed("とりあえずのメモ")),
                        repl::Metadata::WordTags(vec![Cow::Borrowed("取引")]),
                    ],
                    posts: vec![
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                    value: PrettyDecimal::comma3dot(dec!(-123456.78)),
                                    commodity: Cow::Borrowed("USD"),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            metadata: vec![
                                repl::Metadata::Comment(Cow::Borrowed("Note expense A")),
                                repl::Metadata::KeyValueTag {
                                    key: Cow::Borrowed("Payee"),
                                    value: repl::MetadataValue::Text(Cow::Borrowed("Bar"))
                                },
                            ],
                            ..repl::Posting::new("Expense A")
                        },
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                    value: PrettyDecimal::unformatted(dec!(12)),
                                    commodity: Cow::Borrowed("JPY"),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            balance: Some(repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                value: PrettyDecimal::plain(dec!(-1000)),
                                commodity: Cow::Borrowed("CHF"),
                            })),
                            metadata: vec![repl::Metadata::WordTags(vec![
                                Cow::Borrowed("tag1"),
                                Cow::Borrowed("他のタグ")
                            ]),],
                            ..repl::Posting::new("Liabilities B")
                        },
                        repl::Posting {
                            balance: Some(repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                value: PrettyDecimal::unformatted(dec!(0)),
                                commodity: Cow::Borrowed(""),
                            })),
                            metadata: vec![
                                repl::Metadata::Comment(Cow::Borrowed("Cのノート")),
                                repl::Metadata::Comment(Cow::Borrowed("これなんだっけ")),
                            ],

                            ..repl::Posting::new("Assets C")
                        }
                    ],
                    ..repl::Transaction::new(NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(), "")
                }
            )
        );
    }
}
