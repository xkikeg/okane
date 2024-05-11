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

use winnow::{
    ascii::line_ending,
    combinator::{alt, cut_err, fail, peek, preceded, repeat, terminated},
    error::StrContext,
    token::{one_of, tag, take_while},
    PResult, Parser,
};

use self::directive::COMMENT_PREFIX;

#[derive(thiserror::Error, Debug)]
#[error("failed to parse the input: \n{0}")]
pub struct ParseLedgerError(String);

/// Parses the whole ledger file.
pub fn parse_ledger(input: &str) -> Result<Vec<repl::LedgerEntry>, ParseLedgerError> {
    preceded(
        repeat::<_, _, (), _, _>(0.., line_ending::<&str, _>),
        repeat(
            0..,
            terminated(
                parse_ledger_entry,
                repeat::<_, _, (), _, _>(0.., line_ending),
            ),
        ),
    )
    .parse(input)
    .map_err(|e| ParseLedgerError(format!("{}", e)))
}

fn parse_ledger_entry(input: &mut &str) -> PResult<repl::LedgerEntry> {
    // TODO: Consider using dispatch
    alt((
        preceded(
            peek(one_of(COMMENT_PREFIX)),
            cut_err(directive::top_comment.map(repl::LedgerEntry::Comment)),
        ),
        preceded(
            peek(tag("account")),
            cut_err(directive::account_declaration.map(repl::LedgerEntry::Account)),
        ),
        preceded(
            peek(tag("apply")),
            cut_err(directive::apply_tag.map(repl::LedgerEntry::ApplyTag)),
        ),
        preceded(
            peek(tag("commodity")),
            cut_err(directive::commodity_declaration.map(repl::LedgerEntry::Commodity)),
        ),
        preceded(
            peek(tag("end")),
            cut_err(directive::end_apply_tag.map(|_| repl::LedgerEntry::EndApplyTag)),
        ),
        preceded(
            peek(tag("include")),
            cut_err(directive::include.map(repl::LedgerEntry::Include)),
        ),
        preceded(
            peek(take_while(1..=1, |c: char| c.is_ascii_digit())),
            cut_err(transaction::transaction.map(repl::LedgerEntry::Txn)),
        ),
        fail.context(StrContext::Label("no matching syntax")),
    ))
    .parse_next(input)
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
