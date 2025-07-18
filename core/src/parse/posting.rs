//! Parser about ledger postings.

use std::borrow::Cow;

use winnow::{
    ascii::space0,
    combinator::{
        alt, cond, delimited, eof, fail, opt, peek, preceded, repeat_till, terminated, trace,
    },
    error::{AddContext, FromExternalError, ParserError, StrContext},
    stream::{AsChar, Stream, StreamIsPartial},
    token::{literal, one_of, take_till},
    Parser,
};

use crate::{
    parse::{
        character::{line_ending_or_semi, paren},
        combinator::{cond_else, has_peek},
        expr, metadata, primitive,
    },
    syntax::{self, decoration::Decoration},
};

pub fn posting<'i, Deco, Input, E>(
    input: &mut Input,
) -> winnow::Result<syntax::Posting<'i, Deco>, E>
where
    Deco: Decoration,
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input>
        + AddContext<Input, StrContext>
        + FromExternalError<Input, pretty_decimal::ParseError>
        + FromExternalError<Input, chrono::ParseError>,
{
    trace("posting::posting", move |input: &mut Input| {
        let clear_state = preceded(space0, metadata::clear_state).parse_next(input)?;
        let account = posting_account::<Deco, _, _>
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

/// Parses the posting account name.
fn posting_account<'i, Deco, Input, E>(
    input: &mut Input,
) -> winnow::Result<Deco::Decorated<Cow<'i, str>>, E>
where
    Deco: Decoration,
    Input: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input>,
{
    trace(
        "posting::posting",
        terminated(
            Deco::decorate_parser(
                repeat_till(
                    1..,
                    // at most one space, followed by account capable chars.
                    (opt(" "), take_till(1.., b"\n\r; \t")),
                    // stop when you can see double space, or terminate char.
                    // you need opt(" ") for the case like ` \t`.
                    peek(alt((
                        "  ",
                        (opt(" "), one_of(('\t', ';', '\r', '\n'))).take(),
                        eof,
                    ))),
                )
                // let repeat_till accumulated into () (unit)
                .map(|((), _)| ())
                .take()
                .map(|x: &str| Cow::Borrowed(x.trim_start())),
            ),
            space0,
        ),
    )
    .parse_next(input)
}

fn posting_amount<'i, Deco, Input, E>(
    input: &mut Input,
) -> winnow::Result<syntax::PostingAmount<'i, Deco>, E>
where
    Deco: Decoration,
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>
        + std::clone::Clone,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input>
        + AddContext<Input, StrContext>
        + FromExternalError<Input, pretty_decimal::ParseError>
        + FromExternalError<Input, chrono::ParseError>,
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

fn lot<'i, Deco, Input, E>(input: &mut Input) -> winnow::Result<syntax::Lot<'i, Deco>, E>
where
    Deco: Decoration,
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Location
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input>
        + AddContext<Input, StrContext>
        + FromExternalError<Input, pretty_decimal::ParseError>
        + FromExternalError<Input, chrono::ParseError>,
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

fn lot_amount<'i, Input, E>(input: &mut Input) -> winnow::Result<syntax::Exchange<'i>, E>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input> + FromExternalError<Input, pretty_decimal::ParseError>,
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

fn total_cost<'i, Input, E>(input: &mut Input) -> winnow::Result<syntax::Exchange<'i>, E>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input> + FromExternalError<Input, pretty_decimal::ParseError>,
{
    preceded((literal("@@"), space0), expr::value_expr)
        .map(syntax::Exchange::Total)
        .parse_next(input)
}

fn rate_cost<'i, Input, E>(input: &mut Input) -> winnow::Result<syntax::Exchange<'i>, E>
where
    Input: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + Clone,
    <Input as Stream>::Token: AsChar + Clone,
    E: ParserError<Input> + FromExternalError<Input, pretty_decimal::ParseError>,
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
    use pretty_decimal::PrettyDecimal;
    use rust_decimal_macros::dec;
    use syntax::plain;
    use winnow::{error::ContextError, LocatingSlice};

    use crate::{
        parse::testing::expect_parse_ok,
        syntax::plain::{Lot, Posting, PostingAmount},
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
                    ..Posting::new_untracked("Expenses:Commissions")
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
                    ..Posting::new_untracked("Expenses")
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
                    ..Posting::new_untracked("Expenses")
                }
            )
        );
    }

    fn posting_account<'a, E>(input: &mut LocatingSlice<&'a str>) -> winnow::Result<Cow<'a, str>, E>
    where
        E: ParserError<LocatingSlice<&'a str>>,
    {
        super::posting_account::<plain::Ident, _, E>(input)
    }

    #[test]
    fn posting_account_returns_minimal() {
        let input = indoc! {"
            Account Value     ;
            next_token
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            (";\nnext_token\n", Cow::Borrowed("Account Value"))
        );
        let input = indoc! {"
            Account Value\t\t
            next_token
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            ("\nnext_token\n", Cow::Borrowed("Account Value"))
        );
        let input = indoc! {"
            Account Value
            next_token
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            ("\nnext_token\n", Cow::Borrowed("Account Value"))
        );
    }

    #[test]
    fn posting_account_unicode() {
        // ends with double spaces
        let input = indoc! {"
            資産:銀行     ;
            next_token
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            (";\nnext_token\n", Cow::Borrowed("資産:銀行"))
        );

        // ends with tab
        let input = indoc! {"
            負債:クレカ\t
            next_token
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            ("\nnext_token\n", Cow::Borrowed("負債:クレカ"))
        );

        // end with the newline
        let input = indoc! {"
            ピカチュウ
            next_token
        "};
        assert_eq!(
            expect_parse_ok(posting_account, input),
            ("\nnext_token\n", Cow::Borrowed("ピカチュウ"))
        );

        // end with the semi-colon
        let input = "ピカチュウ ;next_token";
        assert_eq!(
            expect_parse_ok(posting_account, input),
            (";next_token", Cow::Borrowed("ピカチュウ"))
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
                        preceded(space0, lot::<plain::Ident, _, ContextError<_>>)
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
