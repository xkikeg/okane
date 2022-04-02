// Defines parser for the Ledger format.

use crate::repl;

use std::cmp::min;

use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, take, take_till, take_while},
    character::complete::{char, digit1, line_ending, one_of, space0, space1},
    combinator::{cond, eof, map, map_res, opt, peek, recognize},
    error::{convert_error, FromExternalError, ParseError, VerboseError},
    multi::{many0, many_till},
    sequence::{delimited, preceded, terminated, tuple},
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
    let (input, payee) = terminated(take_till(is_line_ending), line_ending)(input)?;
    let (input, (posts, _)) = many_till(parse_posting, line_ending_or_eof)(input)?;
    Ok((
        input,
        repl::Transaction {
            effective_date,
            clear_state,
            code: code.map(str::to_string),
            posts,
            ..repl::Transaction::new(date, payee.to_string())
        },
    ))
}

fn parse_posting(input: &str) -> IResult<&str, repl::Post, VerboseError<&str>> {
    let (input, account) = preceded(space1, parse_posting_account)(input)?;
    let (input, no_amount) = peek(map(opt(line_ending), |c| c.is_some()))(input)?;
    if no_amount {
        let (input, _) = line_ending(input)?;
        return Ok((
            input,
            repl::Post {
                ..repl::Post::new(account.to_string())
            },
        ));
    }
    let (input, _) = space1(input)?;
    let (input, amount) = opt(map(parse_amount, |amount| repl::ExchangedAmount {
        amount,
        exchange: None,
    }))(input)?;
    let (input, _) = line_ending(input)?;
    Ok((
        input,
        repl::Post {
            amount,
            ..repl::Post::new(account.to_string())
        },
    ))
}

fn parse_posting_account<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    let (input, line) = peek(take_till(is_line_ending))(input)?;
    let space = line.find("  ");
    let tab = line.find('\t');
    let length = match (space, tab) {
        (Some(x), Some(y)) => min(x, y),
        (Some(x), None) => x,
        (None, Some(x)) => x,
        _ => line.len(),
    };
    take(length)(input)
}

fn parse_amount<'a>(input: &'a str) -> IResult<&str, repl::Amount, VerboseError<&'a str>> {
    // Currently it only supports suffix commodity.
    // It should support prefix like $, € or ¥ prefix.
    let (input, value) = terminated(
        map_res(is_a("-0123456789,."), repl::parse_comma_decimal),
        space0,
    )(input)?;
    let (input, c) = terminated(parse_commodity, space0)(input)?;
    Ok((
        input,
        repl::Amount {
            value,
            commodity: c.to_string(),
        },
    ))
}

fn parse_commodity<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    // Quoted commodity not supported.
    is_not(" \t\r\n0123456789.,;:?!-+*/^&|=<>[](){}@")(input)
}

fn parse_paren_str<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    paren(take_while(|c| c != ')'))(input)
}

fn is_line_ending(c: char) -> bool {
    c == '\r' || c == '\n'
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
        let input = indoc! {"

            2022/01/23
        "};
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
            2022/01/23=2022/01/28 ! (code) Foo
             Expense A\t\t-123,456.78 USD
             Liabilities B  12 JPY
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
                    posts: vec![
                        repl::Post {
                            amount: Some(repl::ExchangedAmount {
                                amount: repl::Amount {
                                    value: dec!(-123456.78),
                                    commodity: "USD".to_string(),
                                },
                                exchange: None,
                            }),
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
                            ..repl::Post::new("Liabilities B".to_string())
                        },
                    ],
                    ..repl::Transaction::new(NaiveDate::from_ymd(2022, 1, 23), "".to_string())
                }
            )
        );
    }

    #[test]
    fn parse_posting_account_returns_minimal() {
        let input = indoc! {"
            Account Value     ;
            Next Account Value
        "};
        assert_eq!(
            run_parse(parse_posting_account, input),
            ("     ;\nNext Account Value\n", "Account Value")
        );
        let input = indoc! {"
            Account Value\t\t
            Next Account Value
        "};
        assert_eq!(
            run_parse(parse_posting_account, input),
            ("\t\t\nNext Account Value\n", "Account Value")
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
