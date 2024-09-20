//! Defines parser functions for transaction.

use std::borrow::Cow;

use winnow::{
    ascii::{space0, space1},
    combinator::{cond, cut_err, opt, peek, preceded, repeat, terminated, trace},
    error::StrContext,
    stream::{AsChar, Stream, StreamIsPartial},
    token::one_of,
    PResult, Parser,
};

use crate::{
    parse::{character, combinator::has_peek, metadata, posting, primitive},
    syntax::{self, decoration::Decoration},
};

/// Parses a transaction from given string.
pub fn transaction<'i, I, Deco>(input: &mut I) -> PResult<syntax::Transaction<'i, Deco>>
where
    I: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>
        + winnow::stream::Location
        + Clone,
    <I as Stream>::Token: AsChar + Clone,
    Deco: Decoration,
{
    trace("transaction::transaction", move |input: &mut I| {
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
        let posts = repeat(
            0..,
            Deco::decorate_parser(preceded(peek(one_of(b" \t")), cut_err(posting::posting))),
        )
        .parse_next(input)?;
        Ok(syntax::Transaction {
            effective_date,
            clear_state,
            code: code.map(Cow::Borrowed),
            posts,
            metadata,
            ..syntax::Transaction::new(date, payee.unwrap_or(""))
        })
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::{
        parse::testing::expect_parse_ok,
        syntax::{
            plain::{Posting, PostingAmount, Transaction},
            pretty_decimal::PrettyDecimal,
        },
    };

    #[test]
    fn transaction_parses_valid_minimal() {
        let input = "2022/01/23\n";
        assert_eq!(
            expect_parse_ok(transaction, input),
            (
                "",
                Transaction::new(NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(), "")
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
                syntax::Transaction {
                    effective_date: Some(NaiveDate::from_ymd_opt(2022, 1, 28).unwrap()),
                    clear_state: syntax::ClearState::Cleared,
                    code: Some(Cow::Borrowed("code")),
                    payee: Cow::Borrowed("Foo"),
                    posts: vec![
                        Posting {
                            amount: Some(PostingAmount {
                                amount: syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                                    value: PrettyDecimal::comma3dot(dec!(123456.78)),
                                    commodity: Cow::Borrowed("USD"),
                                }),
                                cost: None,
                                lot: syntax::Lot::default(),
                            }),
                            ..Posting::new("Expense A")
                        },
                        Posting::new("Liabilities B")
                    ],
                    ..Transaction::new(NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(), "")
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
                syntax::Transaction {
                    effective_date: Some(NaiveDate::from_ymd_opt(2022, 1, 28).unwrap()),
                    clear_state: syntax::ClearState::Pending,
                    code: Some(Cow::Borrowed("code")),
                    payee: Cow::Borrowed("Foo"),
                    metadata: vec![
                        syntax::Metadata::Comment(Cow::Borrowed("とりあえずのメモ")),
                        syntax::Metadata::WordTags(vec![Cow::Borrowed("取引")]),
                    ],
                    posts: vec![
                        Posting {
                            amount: Some(PostingAmount {
                                amount: syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                                    value: PrettyDecimal::comma3dot(dec!(-123456.78)),
                                    commodity: Cow::Borrowed("USD"),
                                }),
                                cost: None,
                                lot: syntax::Lot::default(),
                            }),
                            metadata: vec![
                                syntax::Metadata::Comment(Cow::Borrowed("Note expense A")),
                                syntax::Metadata::KeyValueTag {
                                    key: Cow::Borrowed("Payee"),
                                    value: syntax::MetadataValue::Text(Cow::Borrowed("Bar"))
                                },
                            ],
                            ..Posting::new("Expense A")
                        },
                        Posting {
                            amount: Some(PostingAmount {
                                amount: syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                                    value: PrettyDecimal::unformatted(dec!(12)),
                                    commodity: Cow::Borrowed("JPY"),
                                }),
                                cost: None,
                                lot: syntax::Lot::default(),
                            }),
                            balance: Some(syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                                value: PrettyDecimal::plain(dec!(-1000)),
                                commodity: Cow::Borrowed("CHF"),
                            })),
                            metadata: vec![syntax::Metadata::WordTags(vec![
                                Cow::Borrowed("tag1"),
                                Cow::Borrowed("他のタグ")
                            ]),],
                            ..Posting::new("Liabilities B")
                        },
                        Posting {
                            balance: Some(syntax::expr::ValueExpr::Amount(syntax::expr::Amount {
                                value: PrettyDecimal::unformatted(dec!(0)),
                                commodity: Cow::Borrowed(""),
                            })),
                            metadata: vec![
                                syntax::Metadata::Comment(Cow::Borrowed("Cのノート")),
                                syntax::Metadata::Comment(Cow::Borrowed("これなんだっけ")),
                            ],

                            ..Posting::new("Assets C")
                        }
                    ],
                    ..Transaction::new(NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(), "")
                }
            )
        );
    }
}
