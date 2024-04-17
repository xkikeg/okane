//! Defines parser for the Ledger format.

mod character;
mod combinator;
mod directive;
mod expr;
mod metadata;
mod posting;
mod primitive;
mod transaction;

#[cfg(test)]
pub mod testing;

use crate::repl;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{line_ending, one_of},
    combinator::{cut, eof, fail, map, peek},
    error::{context, convert_error, VerboseError},
    multi::{many0, many_till},
    sequence::{preceded, terminated},
    Finish, IResult,
};

#[derive(thiserror::Error, Debug)]
#[error("failed to parse the input: \n{0}")]
pub struct ParseLedgerError(String);

/// Parses the whole ledger file.
pub fn parse_ledger(input: &str) -> Result<Vec<repl::LedgerEntry>, ParseLedgerError> {
    match preceded(
        many0(line_ending),
        many_till(terminated(parse_ledger_entry, many0(line_ending)), eof),
    )(input)
    .finish()
    {
        Ok((_, (ret, _))) => Ok(ret),
        Err(e) => Err(ParseLedgerError(convert_error(input, e))),
    }
}

fn parse_ledger_entry(input: &str) -> IResult<&str, repl::LedgerEntry, VerboseError<&str>> {
    alt((
        preceded(
            peek(one_of(";#%|*")),
            cut(map(directive::top_comment, repl::LedgerEntry::Comment)),
        ),
        preceded(
            peek(tag("account")),
            cut(map(
                directive::account_declaration,
                repl::LedgerEntry::Account,
            )),
        ),
        preceded(
            peek(tag("apply")),
            cut(map(directive::apply_tag, repl::LedgerEntry::ApplyTag)),
        ),
        preceded(
            peek(tag("commodity")),
            cut(map(
                directive::commodity_declaration,
                repl::LedgerEntry::Commodity,
            )),
        ),
        preceded(
            peek(tag("end")),
            cut(map(directive::end_apply_tag, |_| {
                repl::LedgerEntry::EndApplyTag
            })),
        ),
        preceded(
            peek(tag("include")),
            cut(map(directive::include, repl::LedgerEntry::Include)),
        ),
        preceded(
            peek(take_while_m_n(1, 1, |c: char| c.is_ascii_digit())),
            cut(map(transaction::transaction, repl::LedgerEntry::Txn)),
        ),
        context("no matching syntax", fail),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_ledger_skips_empty_lines() {
        let input = "\n\n2022/01/23\n";
        assert_eq!(input.chars().next(), Some('\n'));
        assert_eq!(
            parse_ledger(input).unwrap(),
            vec![repl::LedgerEntry::Txn(repl::Transaction::new(
                NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                "".to_string()
            ))]
        );
    }

    #[test]
    fn parse_ledger_two_contiguous_transactions() {
        let input = indoc! {"
            2024/4/10 Migros
                Expenses:Grocery
            2024/4/20 Coop
                Expenses:Grocery
        "};

        assert_eq!(
            parse_ledger(input).unwrap(),
            vec![
                repl::LedgerEntry::Txn(repl::Transaction {
                    posts: vec![repl::Posting::new("Expenses:Grocery".to_string())],
                    ..repl::Transaction::new(
                        NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
                        "Migros".to_string(),
                    )
                }),
                repl::LedgerEntry::Txn(repl::Transaction {
                    posts: vec![repl::Posting::new("Expenses:Grocery".to_string())],
                    ..repl::Transaction::new(
                        NaiveDate::from_ymd_opt(2024, 4, 20).unwrap(),
                        "Coop".to_string(),
                    )
                })
            ]
        )
    }
}
