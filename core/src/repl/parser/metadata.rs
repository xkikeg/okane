//! Parsers related to metadata aka comments.

use crate::repl;
use repl::parser::{character, combinator::has_peek};

use winnow::{
    branch::alt,
    bytes::{one_of, tag, take_till1},
    character::{line_ending, not_line_ending, space0, space1},
    combinator::{cond, cut_err},
    error::{ContextError, ParseError},
    multi::{many1, separated0},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

/// Parses block of metadata including the last line_end.
/// Note this consumes one line_ending regardless of Metadata existence.
pub fn block_metadata<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&str, Vec<repl::Metadata>, E> {
    let (input, is_metadata) = has_peek(one_of(';')).parse_next(input)?;
    let (input, _) = cond(!is_metadata, line_ending)(input)?;
    separated0(preceded(space0, line_metadata), space1)(input)
}

fn line_metadata<'a, E>(input: &'a str) -> IResult<&'a str, repl::Metadata, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    delimited(
        (one_of(';'), space0),
        alt((
            metadata_tags,
            metadata_kv,
            not_line_ending.map(|s: &str| {
                if s.contains(':') {
                    log::warn!("metadata containing `:` not parsed as tags: {}", s);
                }
                repl::Metadata::Comment(s.trim_end().to_string())
            }),
        )),
        character::line_ending_or_eof,
    )
    .context("parsing a line for Metadata")
    .parse_next(input)
}

fn metadata_tags<'a, E>(input: &'a str) -> IResult<&'a str, repl::Metadata, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (input, tags): (_, Vec<&str>) =
        delimited(one_of(':'), many1(terminated(tag_key, one_of(':'))), space0)(input)?;
    Ok((
        input,
        repl::Metadata::WordTags(tags.into_iter().map(String::from).collect()),
    ))
}

fn metadata_kv<'a, E>(input: &'a str) -> IResult<&'a str, repl::Metadata, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    (terminated(tag_key, space0), metadata_value)
        .map(|(key, value)| repl::Metadata::KeyValueTag {
            key: key.to_string(),
            value,
        })
        .parse_next(input)
}

/// Parses metadata value with `:` or `::` prefix.
pub fn metadata_value<'a, E>(input: &'a str) -> IResult<&'a str, repl::MetadataValue, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let expr = preceded(tag("::"), cut_err(not_line_ending))
        .map(|x: &'a str| repl::MetadataValue::Expr(x.trim().to_string()));
    let text = preceded(one_of(':'), cut_err(not_line_ending))
        .map(|x: &'a str| repl::MetadataValue::Text(x.trim().to_string()));
    alt((expr, text))(input)
}

/// Parses metadata tag.
pub fn tag_key<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    take_till1(|c: char| c.is_whitespace() || c == ':')
        .context("metadata tag")
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
