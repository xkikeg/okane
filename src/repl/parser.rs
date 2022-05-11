// Defines parser for the Ledger format.

use crate::repl;

use std::cmp::min;

use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, take, take_till1, take_while},
    character::complete::{char, digit1, line_ending, not_line_ending, one_of, space0, space1},
    combinator::{cond, eof, map, map_res, opt, peek, recognize},
    error::{context, convert_error, ContextError, FromExternalError, ParseError, VerboseError},
    multi::{many0, many1, many_till, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, IResult, Parser,
};

#[derive(thiserror::Error, Debug)]
#[error("failed to parse the input: \n{0}")]
pub struct ParseLedgerError(String);

/// Parses the whole ledger file.
pub fn parse_ledger(input: &str) -> Result<Vec<repl::LedgerEntry>, ParseLedgerError> {
    match many_till(parse_ledger_entry, eof)(input).finish() {
        Ok((_, (ret, _))) => Ok(ret),
        Err(e) => Err(ParseLedgerError(convert_error(input, e))),
    }
}

fn parse_ledger_entry(input: &str) -> IResult<&str, repl::LedgerEntry, VerboseError<&str>> {
    map(
        preceded(many0(line_ending), parse_transaction),
        repl::LedgerEntry::Txn,
    )(input)
}

/// Parses a transaction from given string.
fn parse_transaction(input: &str) -> IResult<&str, repl::Transaction, VerboseError<&str>> {
    let (input, date) = parse_date(input)?;
    let (input, effective_date) = opt(preceded(char('='), parse_date))(input)?;
    let (input, is_shortest) = peek(opt(line_ending_or_eof))(input)?;
    // Date (and effective date) should be followed by space, unless followed by line_ending.
    let (input, _) = cond(is_shortest.is_none(), space1)(input)?;
    let (input, cs) = opt(terminated(one_of("*!"), space0))(input)?;
    let clear_state = match cs {
        None => repl::ClearState::Uncleared,
        Some('*') => repl::ClearState::Cleared,
        Some('!') => repl::ClearState::Pending,
        Some(unknown) => unreachable!("unaceptable ClearState {}", unknown),
    };
    let (input, code) = opt(terminated(parse_paren_str, space0))(input)?;
    let (input, payee) = opt(map(not_line_ending_or_comma, str::trim_end))(input)?;
    let (input, metadata) = parse_block_metadata(input)?;
    let (input, (posts, _)) = many_till(parse_posting, line_ending_or_eof)(input)?;
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

fn parse_posting(input: &str) -> IResult<&str, repl::Post, VerboseError<&str>> {
    context("posting of the transaction", |input| {
        let (input, account) = preceded(space1, parse_posting_account)(input)?;
        let (input, has_amount) = peek(map(opt(line_ending_or_comma), |c| c.is_none()))(input)?;
        if !has_amount {
            let (input, metadata) = parse_block_metadata(input)?;
            return Ok((
                input,
                repl::Post {
                    metadata,
                    ..repl::Post::new(account.to_string())
                },
            ));
        }
        let (input, amount) = context(
            "amount of the posting",
            opt(map(terminated(parse_amount, space0), |amount| {
                repl::ExchangedAmount {
                    amount,
                    exchange: None,
                }
            })),
        )(input)?;
        let (input, balance) = context(
            "balance of the posting",
            opt(delimited(pair(char('='), space0), parse_amount, space0)),
        )(input)?;
        let (input, metadata) = parse_block_metadata(input)?;
        Ok((
            input,
            repl::Post {
                amount,
                balance,
                metadata,
                ..repl::Post::new(account.to_string())
            },
        ))
    })(input)
}

/// Parses the posting account name, and consumes the trailing spaces and tabs.
fn parse_posting_account<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    let (input, line) = peek(not_line_ending_or_comma)(input)?;
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

fn parse_amount<'a>(input: &'a str) -> IResult<&str, repl::Amount, VerboseError<&'a str>> {
    // Currently it only supports suffix commodity.
    // It should support prefix like $, € or ¥ prefix.
    let (input, value) = terminated(
        map_res(is_a("-0123456789,."), repl::parse_comma_decimal),
        space0,
    )(input)?;
    let (input, c) = parse_commodity(input)?;
    Ok((
        input,
        repl::Amount {
            value,
            commodity: c.to_string(),
        },
    ))
}

/// Parses block of metadata including the last line_end.
/// Note this consumes one line_ending regardless of Metadata existence.
fn parse_block_metadata<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, Vec<repl::Metadata>, E> {
    let (input, next) = peek(opt(char(';')))(input)?;
    let (input, _) = cond(next.is_none(), line_ending)(input)?;
    // Eats the first space in case next is None (no inline comment).
    separated_list0(space1, preceded(space0, parse_line_metadata))(input)
}

fn parse_line_metadata<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, repl::Metadata, E> {
    context(
        "parsing a line for Metadata",
        delimited(
            pair(char(';'), space0),
            alt((
                parse_metadata_tags,
                parse_metadata_kv,
                map(not_line_ending, |s: &str| {
                    if s.contains(':') {
                        log::warn!("metadata containing `:` not parsed as tags");
                    }
                    repl::Metadata::Comment(s.trim_end().to_string())
                }),
            )),
            line_ending_or_eof,
        ),
    )(input)
}

fn parse_metadata_tags<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&str, repl::Metadata, E> {
    let (input, tags) = delimited(
        char(':'),
        many1(terminated(
            take_till1(|c: char| c.is_whitespace() || c == ':'),
            char(':'),
        )),
        space0,
    )(input)?;
    Ok((
        input,
        repl::Metadata::WordTags(tags.into_iter().map(String::from).collect()),
    ))
}

fn parse_metadata_kv<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&str, repl::Metadata, E> {
    let (input, (key, value)) = pair(
        terminated(
            take_till1(|c: char| c.is_whitespace() || c == ':'),
            pair(space0, char(':')),
        ),
        preceded(space0, not_line_ending),
    )(input)?;
    Ok((
        input,
        repl::Metadata::KeyValueTag {
            key: key.to_string(),
            value: value.trim_end().to_string(),
        },
    ))
}

fn parse_commodity<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    // Quoted commodity not supported.
    map(
        opt(is_not(" \t\r\n0123456789.,;:?!-+*/^&|=<>[](){}@")),
        |x| x.unwrap_or_default(),
    )(input)
}

fn parse_paren_str<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    paren(take_while(|c| c != ')'))(input)
}

fn line_ending_or_comma<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    alt((line_ending, recognize(char(';'))))(input)
}

/// Parses non-zero string until line_ending or comma appears.
fn not_line_ending_or_comma<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    is_not(";\r\n")(input)
}

fn line_ending_or_eof<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    alt((eof, line_ending))(input)
}

fn paren<I, O, E: ParseError<I>, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: Parser<I, O, E>,
    I: nom::Slice<core::ops::RangeFrom<usize>> + nom::InputIter,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    delimited(char('('), inner, char(')'))
}

fn parse_date<'a, E: ParseError<&'a str> + FromExternalError<&'a str, chrono::ParseError>>(
    input: &'a str,
) -> IResult<&str, NaiveDate, E> {
    map_res(
        recognize(tuple((digit1, char('/'), digit1, char('/'), digit1))),
        |s| NaiveDate::parse_from_str(s, "%Y/%m/%d"),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    fn run_parse<I, O, F>(mut parser: F, input: I) -> (I, O)
    where
        I: std::ops::Deref<Target = str> + std::fmt::Display + Copy,
        F: Parser<I, O, VerboseError<I>>,
    {
        match parser.parse(input) {
            Ok(res) => res,
            Err(e) => match e {
                nom::Err::Incomplete(_) => panic!("failed with incomplete: input: {}", input),
                nom::Err::Error(e) => panic!("error: {}", convert_error(input, e)),
                nom::Err::Failure(e) => panic!("error: {}", convert_error(input, e)),
            },
        }
    }

    #[test]
    fn parse_ledger_skips_empty_lines() {
        let input = "\n\n2022/01/23\n";
        assert_eq!(input.chars().next(), Some('\n'));
        assert_eq!(
            parse_ledger(input).unwrap(),
            vec![repl::LedgerEntry::Txn(repl::Transaction::new(
                NaiveDate::from_ymd(2022, 1, 23),
                "".to_string()
            ))]
        );
    }

    #[test]
    fn parse_transaction_valid_minimal() {
        let input = "2022/01/23\n";
        assert_eq!(
            run_parse(parse_transaction, input),
            (
                "",
                repl::Transaction::new(NaiveDate::from_ymd(2022, 1, 23), "".to_string())
            )
        );
    }

    #[test]
    fn parse_transaction_valid_compact() {
        let input = indoc! {"
            2022/01/23=2022/01/28 *(code)Foo
             Expense A\t123,456.78 USD
             Liabilities B
        "};
        assert_eq!(
            run_parse(parse_transaction, input),
            (
                "",
                repl::Transaction {
                    effective_date: Some(NaiveDate::from_ymd(2022, 1, 28)),
                    clear_state: repl::ClearState::Cleared,
                    code: Some("code".to_string()),
                    payee: "Foo".to_string(),
                    posts: vec![
                        repl::Post {
                            amount: Some(repl::ExchangedAmount {
                                amount: repl::Amount {
                                    value: dec!(123456.78),
                                    commodity: "USD".to_string(),
                                },
                                exchange: None,
                            }),
                            ..repl::Post::new("Expense A".to_string())
                        },
                        repl::Post::new("Liabilities B".to_string())
                    ],
                    ..repl::Transaction::new(NaiveDate::from_ymd(2022, 1, 23), "".to_string())
                }
            )
        );
    }

    #[test]
    fn parse_transaction_valid_complex() {
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
            run_parse(parse_transaction, input),
            (
                "",
                repl::Transaction {
                    effective_date: Some(NaiveDate::from_ymd(2022, 1, 28)),
                    clear_state: repl::ClearState::Pending,
                    code: Some("code".to_string()),
                    payee: "Foo".to_string(),
                    metadata: vec![
                        repl::Metadata::Comment("とりあえずのメモ".to_string()),
                        repl::Metadata::WordTags(vec!["取引".to_string()]),
                    ],
                    posts: vec![
                        repl::Post {
                            amount: Some(repl::ExchangedAmount {
                                amount: repl::Amount {
                                    value: dec!(-123456.78),
                                    commodity: "USD".to_string(),
                                },
                                exchange: None,
                            }),
                            metadata: vec![
                                repl::Metadata::Comment("Note expense A".to_string()),
                                repl::Metadata::KeyValueTag {
                                    key: "Payee".to_string(),
                                    value: "Bar".to_string()
                                },
                            ],
                            ..repl::Post::new("Expense A".to_string())
                        },
                        repl::Post {
                            amount: Some(repl::ExchangedAmount {
                                amount: repl::Amount {
                                    value: dec!(12),
                                    commodity: "JPY".to_string(),
                                },
                                exchange: None,
                            }),
                            balance: Some(repl::Amount {
                                value: dec!(-1000),
                                commodity: "CHF".to_string(),
                            }),
                            metadata: vec![repl::Metadata::WordTags(vec![
                                "tag1".to_string(),
                                "他のタグ".to_string()
                            ]),],
                            ..repl::Post::new("Liabilities B".to_string())
                        },
                        repl::Post {
                            balance: Some(repl::Amount {
                                value: dec!(0),
                                commodity: "".to_string(),
                            }),
                            metadata: vec![
                                repl::Metadata::Comment("Cのノート".to_string()),
                                repl::Metadata::Comment("これなんだっけ".to_string()),
                            ],

                            ..repl::Post::new("Assets C".to_string())
                        }
                    ],
                    ..repl::Transaction::new(NaiveDate::from_ymd(2022, 1, 23), "".to_string())
                }
            )
        );
    }

    #[test]
    fn parse_posting_many_comments() {
        let input: &str = " Expenses:Commissions    1 USD ; Payee: My Card\n ; My card took commission\n ; :financial:経済:\n";
        assert_eq!(
            run_parse(parse_posting, input),
            (
                "",
                repl::Post {
                    amount: Some(repl::ExchangedAmount {
                        amount: repl::Amount {
                            value: dec!(1),
                            commodity: "USD".to_string(),
                        },
                        exchange: None,
                    },),
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
                    ..repl::Post::new("Expenses:Commissions".to_string())
                }
            )
        )
    }

    #[test]
    fn parse_posting_account_returns_minimal() {
        let input = indoc! {"
            Account Value     ;
            Next Account Value
        "};
        assert_eq!(
            run_parse(parse_posting_account, input),
            (";\nNext Account Value\n", "Account Value")
        );
        let input = indoc! {"
            Account Value\t\t
            Next Account Value
        "};
        assert_eq!(
            run_parse(parse_posting_account, input),
            ("\nNext Account Value\n", "Account Value")
        );
        let input = indoc! {"
            Account Value
            Next Account Value
        "};
        assert_eq!(
            run_parse(parse_posting_account, input),
            ("\nNext Account Value\n", "Account Value")
        );
    }

    #[test]
    fn parse_line_metadata_valid_tags() {
        let input: &str = ";   :tag1:tag2:tag3:\n";
        assert_eq!(
            run_parse(parse_line_metadata, input),
            (
                "",
                repl::Metadata::WordTags(vec![
                    "tag1".to_string(),
                    "tag2".to_string(),
                    "tag3".to_string()
                ])
            )
        )
    }

    #[test]
    fn parse_line_metadata_valid_kv() {
        let input: &str = ";   場所: ドラッグストア\n";
        assert_eq!(
            run_parse(parse_line_metadata, input),
            (
                "",
                repl::Metadata::KeyValueTag {
                    key: "場所".to_string(),
                    value: "ドラッグストア".to_string(),
                }
            )
        )
    }

    #[test]
    fn parse_line_metadata_valid_comment() {
        let input: &str = ";A fox jumps over: この例文見飽きた    \n";
        assert_eq!(
            run_parse(parse_line_metadata, input),
            (
                "",
                repl::Metadata::Comment("A fox jumps over: この例文見飽きた".to_string())
            )
        )
    }

    #[test]
    fn parse_date_valid() {
        let res = run_parse(parse_date, "2022/01/15");
        assert_eq!(res, ("", NaiveDate::from_ymd(2022, 1, 15)));
    }

    #[test]
    fn parse_date_invalid() {
        let pd = parse_date::<nom::error::Error<&'static str>>;
        pd("not a date").unwrap_err();
        pd("2022/01").unwrap_err();
        pd("2022/13/21").unwrap_err();
    }
}
