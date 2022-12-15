//! Parsers related to metadata aka comments.

use crate::repl;
use repl::parser::{character, combinator::has_peek};

use nom::{
    branch::alt,
    bytes::complete::{is_a, take_till1},
    character::complete::{char, line_ending, not_line_ending, space0, space1},
    combinator::{cond, map},
    error::{context, ContextError, ParseError},
    multi::{fold_many1, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

/// Parses top level comment in the Ledger file format.
/// Notable difference with block_metadata is, this accepts multiple prefix.
pub fn top_comment<'a, E>(input: &'a str) -> IResult<&'a str, repl::TopLevelComment, E>
where
    E: ParseError<&'a str>,
{
    map(
        fold_many1(
            delimited(is_a(";#%|*"), not_line_ending, line_ending),
            || String::new(),
            |mut ret, l| {
                ret.push_str(l);
                ret.push('\n');
                ret
            },
        ),
        repl::TopLevelComment,
    )(input)
}

/// Parses block of metadata including the last line_end.
/// Note this consumes one line_ending regardless of Metadata existence.
pub fn block_metadata<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, Vec<repl::Metadata>, E> {
    let (input, is_metadata) = has_peek(char(';'))(input)?;
    let (input, _) = cond(!is_metadata, line_ending)(input)?;
    separated_list0(space1, preceded(space0, line_metadata))(input)
}

fn line_metadata<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
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
            character::line_ending_or_eof,
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;

    #[test]
    fn parse_line_metadata_valid_tags() {
        let input: &str = ";   :tag1:tag2:tag3:\n";
        assert_eq!(
            expect_parse_ok(line_metadata, input),
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
            expect_parse_ok(line_metadata, input),
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
            expect_parse_ok(line_metadata, input),
            (
                "",
                repl::Metadata::Comment("A fox jumps over: この例文見飽きた".to_string())
            )
        )
    }
}
