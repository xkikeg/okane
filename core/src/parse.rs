//! Defines parser for the Ledger format.

mod character;
mod combinator;
mod directive;
mod error;
mod expr;
mod metadata;
mod posting;
mod primitive;
mod transaction;

#[cfg(test)]
pub(crate) mod testing;
use std::ops::Range;

pub use error::ParseError;

use winnow::{
    combinator::{alt, cut_err, dispatch, fail, peek, preceded, trace},
    error::StrContext,
    stream::{Stream, StreamIsPartial},
    token::{any, literal, take_while},
    Located, PResult, Parser,
};

use crate::repl;

/// Parses single ledger repl value with consuming whitespace.
/// To control the behavior precisely, use [ParseOptions::parse_ledger].
pub fn parse_ledger(
    input: &str,
) -> impl Iterator<Item = Result<(ParsedContext, repl::LedgerEntry), ParseError>> {
    ParseOptions::default().parse_ledger(input)
}

/// Options to control parse behavior.
pub struct ParseOptions {
    error_style: annotate_snippets::Renderer,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            error_style: annotate_snippets::Renderer::plain(),
        }
    }
}

impl ParseOptions {
    /// Sets the given [annotate_snippets::Renderer].
    pub fn with_error_style(mut self, error_style: annotate_snippets::Renderer) -> Self {
        self.error_style = error_style;
        self
    }

    pub fn parse_ledger<'i>(
        &self,
        input: &'i str,
    ) -> impl Iterator<Item = Result<(ParsedContext<'i>, repl::LedgerEntry<'i>), ParseError>> + 'i
    {
        ParsedIter {
            initial: input,
            input: Located::new(input),
            renderer: self.error_style.clone(),
        }
    }
}

/// Context information carrying the metadata of the entry.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedContext<'i> {
    initial: &'i str,
    span: Range<usize>,
}

impl<'i> ParsedContext<'i> {
    /// Computes the starting line number from this context.
    /// Note this function is O(N), not a cheap function.
    pub fn compute_line_start(&self) -> usize {
        error::compute_line_number(self.initial, self.span.start)
    }

    /// Returns the [`str`] slice corresponding to this context.
    pub fn as_str(&self) -> &str {
        self.initial
            .get(self.span.clone())
            .expect("ParsedContext::span must be a valid UTF-8 boundary")
    }
}

/// Iterator to return parsed ledger entry one-by-one.
struct ParsedIter<'i> {
    initial: &'i str,
    input: Located<&'i str>,
    renderer: annotate_snippets::Renderer,
}

impl<'i> Iterator for ParsedIter<'i> {
    type Item = Result<(ParsedContext<'i>, repl::LedgerEntry<'i>), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.input.checkpoint();
        (|| {
            take_while(0.., b"\r\n").parse_next(&mut self.input)?;
            if self.input.is_empty() {
                return Ok(None);
            }
            let (entry, span) = parse_ledger_entry.with_span().parse_next(&mut self.input)?;
            Ok(Some((
                ParsedContext {
                    initial: self.initial,
                    span,
                },
                entry,
            )))
        })()
        .map_err(|e| ParseError::new(self.renderer.clone(), self.initial, self.input, start, e))
        .transpose()
    }
}

/// Parses given `input` into `repl::LedgerEntry`.
fn parse_ledger_entry<'i, I>(input: &mut I) -> PResult<repl::LedgerEntry<'i>>
where
    I: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::FindSlice<(char, char)>
        + winnow::stream::Compare<&'static str>
        + Clone,
{
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

    use repl::LedgerEntry;

    fn parse_ledger_into(input: &str) -> Vec<(ParsedContext, LedgerEntry)> {
        let r: Result<Vec<(ParsedContext, LedgerEntry)>, ParseError> =
            ParseOptions::default().parse_ledger(input).collect();
        match r {
            Ok(x) => x,
            Err(e) => panic!("failed to parse:\n{}", e),
        }
    }

    #[test]
    fn parse_ledger_skips_empty_lines() {
        let input = "\n\n2022/01/23\n";
        assert_eq!(input.chars().next(), Some('\n'));
        assert_eq!(
            parse_ledger_into(input),
            vec![(
                ParsedContext {
                    initial: input,
                    span: 2..13
                },
                repl::LedgerEntry::Txn(repl::Transaction::new(
                    NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                    ""
                ))
            )]
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
            parse_ledger_into(input),
            vec![
                (
                    ParsedContext {
                        initial: input,
                        span: 0..38
                    },
                    repl::LedgerEntry::Txn(repl::Transaction {
                        posts: vec![repl::Posting::new("Expenses:Grocery")],
                        ..repl::Transaction::new(
                            NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
                            "Migros",
                        )
                    })
                ),
                (
                    ParsedContext {
                        initial: input,
                        span: 38..74
                    },
                    repl::LedgerEntry::Txn(repl::Transaction {
                        posts: vec![repl::Posting::new("Expenses:Grocery")],
                        ..repl::Transaction::new(
                            NaiveDate::from_ymd_opt(2024, 4, 20).unwrap(),
                            "Coop"
                        )
                    })
                )
            ]
        )
    }
}
