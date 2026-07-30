//! Defines parser for the Ledger format.
//! Currently only the parser for the entire file format is provieded as public.

mod adaptor;
mod character;
mod combinator;
mod directive;
mod error;
mod expr;
mod metadata;
mod posting;
pub(crate) mod price;
mod primitive;
pub(crate) mod transaction;

#[cfg(test)]
pub(crate) mod testing;

pub use adaptor::{ParseOptions, ParsedContext, ParsedSpan};
pub use error::ParseError;

use winnow::{
    ModalResult, Parser,
    combinator::{alt, cut_err, dispatch, fail, peek, preceded, trace},
    error::StrContext,
    stream::{Stream, StreamIsPartial},
    token::{any, literal},
};

use crate::syntax::{self, decoration::Decoration};

/// Parses Ledger `str` containing a list of [`syntax::LedgerEntry`] into iterator.
/// See [`ParseOptions`] to control its behavior.
pub fn parse_ledger<'i, Deco: 'i + Decoration>(
    options: &ParseOptions,
    input: &'i str,
) -> impl Iterator<Item = Result<(ParsedContext<'i>, syntax::LedgerEntry<'i, Deco>), ParseError>> {
    options
        .parse_repeated(parse_ledger_statement, character::newlines.void(), input)
        .enumerate()
        .map(|(i, parsed)| {
            parsed.map(|(ctx, statement)| {
                // The first entry never gets a blank line before it, regardless
                // of the leading newlines the file may have.
                let separation = if i > 0 && ctx.has_blank_line_before() {
                    syntax::Separation::BlankLine
                } else {
                    syntax::Separation::Immediate
                };
                (
                    ctx,
                    syntax::LedgerEntry {
                        separation,
                        statement,
                    },
                )
            })
        })
}

/// Parses given `input` into [syntax::LedgerStatement].
fn parse_ledger_statement<'i, I, Deco>(
    input: &mut I,
) -> ModalResult<syntax::LedgerStatement<'i, Deco>>
where
    I: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>
        + winnow::stream::Location
        + Clone,
    Deco: Decoration + 'static,
{
    trace(
        "parse_ledger_statement",
        dispatch! {peek(any);
            'a' => alt((
                preceded(
                    peek(literal("account")),
                    cut_err(directive::account_declaration.map(syntax::LedgerStatement::Account)),
                ),
                preceded(
                    peek(literal("apply")),
                    cut_err(directive::apply_tag.map(syntax::LedgerStatement::ApplyTag)),
                ),
            )),
            'c' => directive::commodity_declaration.map(syntax::LedgerStatement::Commodity),
            'e' => directive::end_apply_tag.map(|_| syntax::LedgerStatement::EndApplyTag),
            'i' => directive::include.map(syntax::LedgerStatement::Include),
            c if directive::is_comment_prefix(c) => {
                directive::top_comment.map(syntax::LedgerStatement::Comment)
            },
            c if c.is_ascii_digit() => transaction::transaction.map(syntax::LedgerStatement::Txn),
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

    use syntax::plain::LedgerEntry;
    use syntax::{LedgerStatement, Separation};

    fn parse_ledger_into(input: &'_ str) -> Vec<(ParsedContext<'_>, LedgerEntry<'_>)> {
        let r: Result<Vec<(ParsedContext, LedgerEntry)>, ParseError> =
            parse_ledger(&ParseOptions::default(), input).collect();
        match r {
            Ok(x) => x,
            Err(e) => panic!("failed to parse:\n{}", e),
        }
    }

    /// Returns the [`Separation`] of every parsed entry.
    fn parse_ledger_separations(input: &str) -> Vec<Separation> {
        parse_ledger_into(input)
            .iter()
            .map(|(_, entry)| entry.separation)
            .collect()
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
                    span: 2..13,
                    separator: 0..2,
                },
                LedgerEntry {
                    // Leading newlines must not turn into a leading blank line.
                    separation: Separation::Immediate,
                    statement: LedgerStatement::Txn(syntax::Transaction::new(
                        NaiveDate::from_ymd_opt(2022, 1, 23).unwrap(),
                        ""
                    )),
                }
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
                        span: 0..38,
                        separator: 0..0,
                    },
                    LedgerEntry {
                        separation: Separation::Immediate,
                        statement: LedgerStatement::Txn(syntax::Transaction {
                            posts: vec![syntax::Posting::new_untracked("Expenses:Grocery")],
                            ..syntax::Transaction::new(
                                NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
                                "Migros",
                            )
                        }),
                    }
                ),
                (
                    ParsedContext {
                        initial: input,
                        span: 38..74,
                        separator: 38..38,
                    },
                    LedgerEntry {
                        separation: Separation::Immediate,
                        statement: LedgerStatement::Txn(syntax::Transaction {
                            posts: vec![syntax::Posting::new_untracked("Expenses:Grocery")],
                            ..syntax::Transaction::new(
                                NaiveDate::from_ymd_opt(2024, 4, 20).unwrap(),
                                "Coop"
                            )
                        }),
                    }
                )
            ]
        )
    }

    #[test]
    fn parse_ledger_tracks_separation() {
        let input = indoc! {"
            ; comment right above the transaction
            2024/4/10 Migros
                Expenses:Grocery

            include other.ledger
            include another.ledger



            end apply tag
        "};

        assert_eq!(
            parse_ledger_separations(input),
            vec![
                // Leading comment, first entry of the file.
                Separation::Immediate,
                // Transaction glued to the comment above it.
                Separation::Immediate,
                Separation::BlankLine,
                // Consecutive include directives stay grouped.
                Separation::Immediate,
                // Several blank lines are still just one separation.
                Separation::BlankLine,
            ]
        );
    }
}
