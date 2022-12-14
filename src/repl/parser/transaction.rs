//! Defines parser functions for transaction.

use crate::repl;
use repl::parser::{character, combinator::has_peek, metadata, posting, primitive};

use nom::{
    character::complete::{char, one_of, space0, space1},
    combinator::{cond, map, opt},
    error::VerboseError,
    multi::many_till,
    sequence::{preceded, terminated},
    IResult,
};

/// Parses a transaction from given string.
pub fn transaction(input: &str) -> IResult<&str, repl::Transaction, VerboseError<&str>> {
    let (input, date) = primitive::date(input)?;
    let (input, effective_date) = opt(preceded(char('='), primitive::date))(input)?;
    let (input, is_shortest) = has_peek(character::line_ending_or_eof)(input)?;
    // Date (and effective date) should be followed by space, unless followed by line_ending.
    let (input, _) = cond(!is_shortest, space1)(input)?;
    let (input, cs) = opt(terminated(one_of("*!"), space0))(input)?;
    let clear_state = match cs {
        None => repl::ClearState::Uncleared,
        Some('*') => repl::ClearState::Cleared,
        Some('!') => repl::ClearState::Pending,
        Some(unknown) => unreachable!("unaceptable ClearState {}", unknown),
    };
    let (input, code) = opt(terminated(character::paren_str, space0))(input)?;
    let (input, payee) = opt(map(character::not_line_ending_or_semi, str::trim_end))(input)?;
    let (input, metadata) = metadata::block_metadata(input)?;
    let (input, (posts, _)) = many_till(posting::posting, character::line_ending_or_eof)(input)?;
    Ok((
        input,
        repl::Transaction {
            effective_date,
            clear_state,
            code: code.map(str::to_string),
            posts,
            metadata,
            ..repl::Transaction::new(date, payee.unwrap_or("").to_string())
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

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
                                amount: repl::expr::ValueExpr::Amount(repl::Amount {
                                    value: dec!(123456.78),
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
             Liabilities B  12 JPY  =  -1,000 CHF
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
                                amount: repl::expr::ValueExpr::Amount(repl::Amount {
                                    value: dec!(-123456.78),
                                    commodity: "USD".to_string(),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            metadata: vec![
                                repl::Metadata::Comment("Note expense A".to_string()),
                                repl::Metadata::KeyValueTag {
                                    key: "Payee".to_string(),
                                    value: "Bar".to_string()
                                },
                            ],
                            ..repl::Posting::new("Expense A".to_string())
                        },
                        repl::Posting {
                            amount: Some(repl::PostingAmount {
                                amount: repl::expr::ValueExpr::Amount(repl::Amount {
                                    value: dec!(12),
                                    commodity: "JPY".to_string(),
                                }),
                                cost: None,
                                lot: repl::Lot::default(),
                            }),
                            balance: Some(repl::expr::ValueExpr::Amount(repl::Amount {
                                value: dec!(-1000),
                                commodity: "CHF".to_string(),
                            })),
                            metadata: vec![repl::Metadata::WordTags(vec![
                                "tag1".to_string(),
                                "他のタグ".to_string()
                            ]),],
                            ..repl::Posting::new("Liabilities B".to_string())
                        },
                        repl::Posting {
                            balance: Some(repl::expr::ValueExpr::Amount(repl::Amount {
                                value: dec!(0),
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
