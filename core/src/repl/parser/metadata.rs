//! Parsers related to metadata aka comments.

use crate::repl;
use repl::parser::{character, combinator::has_peek};

use winnow::{
    ascii::{line_ending, space0, space1, till_line_ending},
    combinator::{alt, cond, cut_err, delimited, preceded, repeat, separated, terminated, trace},
    error::ParserError,
    token::{one_of, tag, take_till},
    PResult, Parser,
};

/// Parses block of metadata including the last line_end.
/// Note this consumes one line_ending regardless of Metadata existence.
pub fn block_metadata<'a, E>(input: &mut &'a str) -> PResult<Vec<repl::Metadata>, E>
where
    E: ParserError<&'a str>,
{
    // TODO: Clean this up
    let is_metadata = has_peek(one_of(';')).parse_next(input)?;
    cond(!is_metadata, line_ending).void().parse_next(input)?;
    separated(0.., preceded(space0, line_metadata), space1).parse_next(input)
}

fn line_metadata<'a, E>(input: &mut &'a str) -> PResult<repl::Metadata, E>
where
    E: ParserError<&'a str>,
{
    trace(
        "metadata::line_metadata",
        delimited(
            (one_of(';'), space0),
            alt((
                metadata_tags,
                metadata_kv,
                till_line_ending.map(|s: &str| {
                    if s.contains(':') {
                        log::warn!("metadata containing `:` not parsed as tags: {}", s);
                    }
                    repl::Metadata::Comment(s.trim_end().to_string())
                }),
            )),
            character::line_ending_or_eof,
        ),
    )
    .parse_next(input)
}

fn metadata_tags<'a, E>(input: &mut &'a str) -> PResult<repl::Metadata, E>
where
    E: ParserError<&'a str>,
{
    delimited(
        one_of(':'),
        repeat(1.., terminated(tag_key, one_of(':'))),
        space0,
    )
    .map(|tags: Vec<&str>| repl::Metadata::WordTags(tags.into_iter().map(String::from).collect()))
    .parse_next(input)
}

fn metadata_kv<'a, E>(input: &mut &'a str) -> PResult<repl::Metadata, E>
where
    E: ParserError<&'a str>,
{
    (terminated(tag_key, space0), metadata_value)
        .map(|(key, value)| repl::Metadata::KeyValueTag {
            key: key.to_string(),
            value,
        })
        .parse_next(input)
}

/// Parses metadata value with `:` or `::` prefix.
pub fn metadata_value<'a, E>(input: &mut &'a str) -> PResult<repl::MetadataValue, E>
where
    E: ParserError<&'a str>,
{
    let expr = preceded(tag("::"), cut_err(till_line_ending))
        .map(|x: &'a str| repl::MetadataValue::Expr(x.trim().to_string()));
    let text = preceded(one_of(':'), cut_err(till_line_ending))
        .map(|x: &'a str| repl::MetadataValue::Text(x.trim().to_string()));
    alt((expr, text)).parse_next(input)
}

/// Parses metadata tag.
pub fn tag_key<'a, E>(input: &mut &'a str) -> PResult<&'a str, E>
where
    E: ParserError<&'a str>,
{
    trace(
        "metadata::tag_key",
        take_till(1.., |c: char| c.is_whitespace() || c == ':'),
    )
    .parse_next(input)
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
                    value: repl::MetadataValue::Text("ドラッグストア".to_string()),
                }
            )
        );

        let input: &str = ";   日付:: [2022-01-19] \n";
        assert_eq!(
            expect_parse_ok(line_metadata, input),
            (
                "",
                repl::Metadata::KeyValueTag {
                    key: "日付".to_string(),
                    value: repl::MetadataValue::Expr("[2022-01-19]".to_string()),
                }
            )
        );
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
