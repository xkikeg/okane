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

use winnow::{
    ascii::line_ending,
    combinator::{alt, cut_err, dispatch, fail, peek, preceded, repeat, trace},
    error::StrContext,
    token::{any, literal},
    PResult, Parser,
};

use crate::repl;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    // TODO: Use #[from] appropriately
    #[error("failed to parse the input:\n{0}")]
    ParseFailed(String),
}

/// Parses single ledger repl value with consuming whitespace.
pub fn parse_ledger(input: &str) -> impl Iterator<Item = Result<ParsedLedgerEntry, ParseError>> {
    ParsedIter { input }
}

pub type ParsedLedgerEntry<'i> = repl::LedgerEntry<'i>;

/// Iterator to return parsed ledger entry one-by-one.
struct ParsedIter<'i> {
    input: &'i str,
}

impl<'i> Iterator for ParsedIter<'i> {
    type Item = Result<ParsedLedgerEntry<'i>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        (|| {
            repeat(0.., line_ending).parse_next(&mut self.input)?;
            if self.input.is_empty() {
                return Ok(None);
            }
            parse_ledger_entry.parse_next(&mut self.input).map(Some)
        })()
        .map_err(|e| ParseError::ParseFailed(format!("{}", e)))
        .transpose()
    }
}

/// Parses given `input` into `repl::LedgerEntry`.
fn parse_ledger_entry<'i>(input: &mut &'i str) -> PResult<repl::LedgerEntry<'i>> {
    trace(
        "parse_ledger_entry",
        dispatch! {peek(any);
            'a' => alt((
                preceded(
                    peek(literal("account")),
                    cut_err(directive::account_declaration.map(repl::LedgerEntry::Account)),
                ),
                preceded(
                    peek(literal("apply")),
                    cut_err(directive::apply_tag.map(repl::LedgerEntry::ApplyTag)),
                ),
            )),
            'c' => directive::commodity_declaration.map(repl::LedgerEntry::Commodity),
            'e' => directive::end_apply_tag.map(|_| repl::LedgerEntry::EndApplyTag),
            'i' => directive::include.map(repl::LedgerEntry::Include),
            c if directive::is_comment_prefix(c) => {
                directive::top_comment.map(repl::LedgerEntry::Comment)
            },
            c if c.is_ascii_digit() => transaction::transaction.map(repl::LedgerEntry::Txn),
            _ => fail.context(StrContext::Label("no matching syntax")),
        },
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    fn parse_ledger_into(input: &str) -> Result<Vec<ParsedLedgerEntry>, ParseError> {
        parse_ledger(input).collect()
    }

    #[test]
    fn parse_ledger_skips_empty_lines() {
        let input = "\n\n2022/01/23\n";
        assert_eq!(input.chars().next(), Some('\n'));
        assert_eq!(
            parse_ledger_into(input).unwrap(),
            vec![repl::LedgerEntry::Txn(repl::Transaction::new(
                NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                ""
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
            parse_ledger_into(input).unwrap(),
            vec![
                repl::LedgerEntry::Txn(repl::Transaction {
                    posts: vec![repl::Posting::new("Expenses:Grocery")],
                    ..repl::Transaction::new(
                        NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
                        "Migros",
                    )
                }),
                repl::LedgerEntry::Txn(repl::Transaction {
                    posts: vec![repl::Posting::new("Expenses:Grocery")],
                    ..repl::Transaction::new(NaiveDate::from_ymd_opt(2024, 4, 20).unwrap(), "Coop")
                })
            ]
        )
    }
}
