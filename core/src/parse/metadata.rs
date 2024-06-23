//! Parsers related to metadata aka comments.

use std::borrow::Cow;

use winnow::{
    ascii::{line_ending, space0, space1, till_line_ending},
    combinator::{
        alt, backtrack_err, cut_err, delimited, dispatch, opt, peek, preceded, repeat, separated,
        terminated, trace,
    },
    error::ParserError,
    stream::{AsChar, Stream, StreamIsPartial},
    token::{any, literal, one_of, take_till},
    PResult, Parser,
};

use crate::parse::character;
use crate::repl;

/// Parses a ClearState.
pub fn clear_state<I, E>(input: &mut I) -> PResult<repl::ClearState, E>
where
    I: Stream + StreamIsPartial,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "metadata::clear_state",
        opt(terminated(
            alt((
                one_of('*').value(repl::ClearState::Cleared),
                one_of('!').value(repl::ClearState::Pending),
            )),
            space0,
        ))
        .map(|x| x.unwrap_or_default()),
    )
    .parse_next(input)
}

/// Parses block of metadata including the last line_end.
/// Note this consumes at least one line_ending regardless of Metadata existence.
pub fn block_metadata<'i, I, E>(input: &mut I) -> PResult<Vec<repl::Metadata<'i>>, E>
where
    I: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    // For now, we can't go with regular repeat because it's hard to have a initial value in Accumulate.
    trace(
        "metadata::block_metadata",
        dispatch! {peek(any);
            ';' => separated(1.., line_metadata, space1),
            _ => preceded(line_ending, repeat(0.., preceded(space1, line_metadata))),
        },
    )
    .parse_next(input)
}

fn line_metadata<'i, I, E>(input: &mut I) -> PResult<repl::Metadata<'i>, E>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::FindSlice<(char, char)>
        + winnow::stream::Compare<&'static str>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
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
                    repl::Metadata::Comment(s.trim_end().into())
                }),
            )),
            character::line_ending_or_eof,
        ),
    )
    .parse_next(input)
}

fn metadata_tags<'i, I, E>(input: &mut I) -> PResult<repl::Metadata<'i>, E>
where
    I: Stream<Slice = &'i str> + StreamIsPartial,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "metadata::metadata_tags",
        delimited(
            one_of(':'),
            repeat(1.., terminated(tag_key.map(Cow::Borrowed), one_of(':'))),
            space0,
        )
        .map(repl::Metadata::WordTags),
    )
    .parse_next(input)
}

fn metadata_kv<'i, I, E>(input: &mut I) -> PResult<repl::Metadata<'i>, E>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "metadata::metadata_kv",
        (terminated(tag_key, space0), metadata_value).map(|(key, value): (&str, _)| {
            repl::Metadata::KeyValueTag {
                key: key.into(),
                value,
            }
        }),
    )
    .parse_next(input)
}

/// Parses metadata value with `:` or `::` prefix.
pub fn metadata_value<'i, I, E>(input: &mut I) -> PResult<repl::MetadataValue<'i>, E>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    let expr = preceded(literal("::"), cut_err(till_line_ending))
        .map(|x: &str| repl::MetadataValue::Expr(x.trim().into()));
    let text = preceded(one_of(':'), cut_err(till_line_ending))
        .map(|x: &str| repl::MetadataValue::Text(x.trim().into()));
    trace("metadata::metadata_value", backtrack_err(alt((expr, text)))).parse_next(input)
}

/// Parses metadata tag.
pub fn tag_key< I, E>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "metadata::tag_key",
        take_till(1.., |c: <I as Stream>::Token| {
            let c = c.as_char();
            c.is_ascii_whitespace() || c == ':'
        }),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;

    #[test]
    fn block_metadata_empty() {
        let input = "\r\n ";
        assert_eq!(expect_parse_ok(block_metadata, input), (" ", vec![]))
    }

    #[test]
    fn block_metadata_consumes_first_comment_inline() {
        let input = "; foo\n ; bar\n; baz\n";
        assert_eq!(
            expect_parse_ok(block_metadata, input),
            (
                // This line isn't a block_metadata because it doesn't have preceding spaces.
                "; baz\n",
                vec![
                    repl::Metadata::Comment("foo".into()),
                    repl::Metadata::Comment("bar".into()),
                ]
            )
        )
    }

    #[test]
    fn block_metadata_consumes_newline_immediately() {
        let input = "\n ; foo\n ; bar\n";
        assert_eq!(
            expect_parse_ok(block_metadata, input),
            (
                "",
                vec![
                    repl::Metadata::Comment("foo".into()),
                    repl::Metadata::Comment("bar".into()),
                ]
            )
        )
    }

    #[test]
    fn block_metadata_stops_at_top_level_comment() {
        let input = "\n; foo\n";
        assert_eq!(expect_parse_ok(block_metadata, input), ("; foo\n", vec![]))
    }

    #[test]
    fn parse_line_metadata_valid_tags() {
        let input: &str = ";   :tag1:tag2:tag3:\n";
        assert_eq!(
            expect_parse_ok(line_metadata, input),
            (
                "",
                repl::Metadata::WordTags(vec!["tag1".into(), "tag2".into(), "tag3".into()])
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
                    key: "場所".into(),
                    value: repl::MetadataValue::Text("ドラッグストア".into()),
                }
            )
        );

        let input: &str = ";   日付:: [2022-01-19] \n";
        assert_eq!(
            expect_parse_ok(line_metadata, input),
            (
                "",
                repl::Metadata::KeyValueTag {
                    key: "日付".into(),
                    value: repl::MetadataValue::Expr("[2022-01-19]".into()),
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
                repl::Metadata::Comment("A fox jumps over: この例文見飽きた".into())
            )
        )
    }
}
