//! Defines parser for the Ledger format.

mod character;
mod combinator;
mod expr;
mod metadata;
mod posting;
pub(crate) mod primitive;
mod transaction;

#[cfg(test)]
pub mod testing;

use crate::repl;

use nom::{
    character::complete::line_ending,
    combinator::{eof, map},
    error::{convert_error, VerboseError},
    multi::{many0, many_till},
    sequence::preceded,
    Finish, IResult,
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
        preceded(many0(line_ending), transaction::transaction),
        repl::LedgerEntry::Txn,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
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
}
