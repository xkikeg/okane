//! Defines parser functions for transaction.

use crate::{
    parse::{character, combinator::has_peek, metadata, posting, primitive},
    repl,
};

use winnow::{
    ascii::{space0, space1},
    combinator::{cond, cut_err, opt, peek, preceded, repeat, terminated, trace},
    error::StrContext,
    token::one_of,
    PResult, Parser,
};

/// Parses a transaction from given string.
pub fn transaction(input: &mut &str) -> PResult<repl::Transaction> {
    trace("transaction::transaction", move |input: &mut &str| {
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
            code: code.map(str::to_string),
            posts,
            metadata,
            ..repl::Transaction::new(date, payee.unwrap_or("").to_string())
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
                repl::Transaction::new(
                    NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                    "".to_string()
                )
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
                    code: Some("code".to_string()),
                    payee: "Foo".to_string(),
                    posts: vec![
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                    value: PrettyDecimal::comma3dot(dec!(123456.78)),
                                    commodity: "USD".to_string(),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            ..repl::Posting::new("Expense A".to_string())
                        },
                        repl::Posting::new("Liabilities B".to_string())
                    ],
                    ..repl::Transaction::new(
                        NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                        "".to_string()
                    )
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
                    code: Some("code".to_string()),
                    payee: "Foo".to_string(),
                    metadata: vec![
                        repl::Metadata::Comment("とりあえずのメモ".to_string()),
                        repl::Metadata::WordTags(vec!["取引".to_string()]),
                    ],
                    posts: vec![
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                    value: PrettyDecimal::comma3dot(dec!(-123456.78)),
                                    commodity: "USD".to_string(),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            metadata: vec![
                                repl::Metadata::Comment("Note expense A".to_string()),
                                repl::Metadata::KeyValueTag {
                                    key: "Payee".to_string(),
                                    value: repl::MetadataValue::Text("Bar".to_string())
                                },
                            ],
                            ..repl::Posting::new("Expense A".to_string())
                        },
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                    value: PrettyDecimal::unformatted(dec!(12)),
                                    commodity: "JPY".to_string(),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            balance: Some(repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                value: PrettyDecimal::plain(dec!(-1000)),
                                commodity: "CHF".to_string(),
                            })),
                            metadata: vec![repl::Metadata::WordTags(vec![
                                "tag1".to_string(),
                                "他のタグ".to_string()
                            ]),],
                            ..repl::Posting::new("Liabilities B".to_string())
                        },
                        repl::Posting {
                            balance: Some(repl::expr::ValueExpr::Amount(repl::expr::Amount {
                                value: PrettyDecimal::unformatted(dec!(0)),
                                commodity: "".to_string(),
                            })),
                            metadata: vec![
                                repl::Metadata::Comment("Cのノート".to_string()),
                                repl::Metadata::Comment("これなんだっけ".to_string()),
                            ],

                            ..repl::Posting::new("Assets C".to_string())
                        }
                    ],
                    ..repl::Transaction::new(
                        NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                        "".to_string()
                    )
                }
            )
        );
    }
}
