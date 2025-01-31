use crate::syntax;

use super::{character::line_ending_or_eof, expr, metadata};

use std::borrow::Cow;

use winnow::{
    ascii::{space0, space1, till_line_ending},
    combinator::{alt, delimited, opt, preceded, repeat, terminated, trace},
    error::ParserError,
    stream::{AsChar, Stream, StreamIsPartial},
    token::{literal, take_while},
    ModalResult, Parser,
};

/// Returns true if the given character is comment prefix.
#[inline(always)]
pub(super) fn is_comment_prefix<C: AsChar>(c: C) -> bool {
    matches!(c.as_char(), ';' | '#' | '%' | '|' | '*')
}

/// Parses "account" directive.
pub fn account_declaration<'i, I>(input: &mut I) -> ModalResult<syntax::AccountDeclaration<'i>>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    <I as Stream>::Token: AsChar + Clone,
{
    (
        delimited(
            (literal("account"), space1),
            till_line_ending,
            line_ending_or_eof,
        ),
        // TODO: Consider using dispatch
        // Note nesting many0 would cause parse failure,
        // as many0 would fail if the sub-parser consumes empty input.
        // So make sure no branches in alt would emit zero input as success.
        repeat(
            0..,
            alt((
                multiline_text((space1, take_while(1.., is_comment_prefix)))
                    .map(syntax::AccountDetail::Comment),
                multiline_text((space1, literal("note"), space1)).map(syntax::AccountDetail::Note),
                delimited(
                    (space1, literal("alias"), space1),
                    till_line_ending,
                    line_ending_or_eof,
                )
                .map(|a: &str| syntax::AccountDetail::Alias(a.trim_end().into())),
            )),
        ),
    )
        .map(|(name, details): (&str, _)| syntax::AccountDeclaration {
            name: name.trim_end().into(),
            details,
        })
        .parse_next(input)
}

/// Parses "commodity" directive.
pub fn commodity_declaration<'i, I>(input: &mut I) -> ModalResult<syntax::CommodityDeclaration<'i>>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    <I as Stream>::Token: AsChar + Clone,
{
    (
        delimited(
            (literal("commodity"), space1),
            till_line_ending,
            line_ending_or_eof,
        ),
        // TODO: Consider using dispatch
        // Note nesting many0 would cause parse failure at nom 7,
        // as many0 would fail if the sub-parser consumes empty input.
        // So make sure no branches in alt would success for empty input.
        repeat(
            0..,
            alt((
                multiline_text((space1, take_while(1.., is_comment_prefix)))
                    .map(syntax::CommodityDetail::Comment),
                multiline_text((space1, literal("note"), space1))
                    .map(syntax::CommodityDetail::Note),
                delimited(
                    (space1, literal("alias"), space1),
                    till_line_ending,
                    line_ending_or_eof,
                )
                .map(|a: &str| syntax::CommodityDetail::Alias(a.trim_end().into())),
                delimited(
                    (space1, literal("format"), space1),
                    expr::amount,
                    line_ending_or_eof,
                )
                .map(syntax::CommodityDetail::Format),
            )),
        ),
    )
        .map(
            |(name, details): (&'i str, _)| syntax::CommodityDeclaration {
                name: name.trim_end().into(),
                details,
            },
        )
        .parse_next(input)
}

/// Parses "apply tag" directive.
pub fn apply_tag<'i, I>(input: &mut I) -> ModalResult<syntax::ApplyTag<'i>>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    <I as Stream>::Token: AsChar + Clone,
{
    // TODO: value needs to be supported.
    trace(
        "directive::apply_tag",
        (
            preceded(
                (literal("apply"), space1, literal("tag"), space1),
                metadata::tag_key,
            ),
            delimited(space0, opt(metadata::metadata_value), line_ending_or_eof),
        )
            .map(|(key, value): (&str, _)| syntax::ApplyTag {
                key: key.into(),
                value,
            }),
    )
    .parse_next(input)
}

/// Parses "end apply tag" directive.
///
/// Note:
/// "end" directive has complicated semantics and may allow "end" or "end apply".
/// Also comment requires "end" directive.
/// In the meantime, only "end apply tag" is supported, however,
/// pretty sure it'd be needed to rename and extend this function.
pub fn end_apply_tag<I, E>(input: &mut I) -> ModalResult<<I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial + winnow::stream::Compare<&'static str>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar,
{
    trace(
        "directive::end_apply_tag",
        terminated(
            (
                literal("end"),
                space1,
                literal("apply"),
                space1,
                literal("tag"),
            )
                .take(),
            (space0, line_ending_or_eof),
        ),
    )
    .parse_next(input)
}

/// Parses include directive.
/// Note given we'll always have UTF-8 input,
/// we're not using PathBuf but String for the path.
pub fn include<'i, I, E>(input: &mut I) -> ModalResult<syntax::IncludeFile<'i>, E>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::FindSlice<(char, char)>
        + winnow::stream::Compare<&'static str>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "directive::include",
        delimited(
            (literal("include"), space1),
            till_line_ending,
            line_ending_or_eof,
        )
        .map(|x: &str| syntax::IncludeFile(x.trim_end().into())),
    )
    .parse_next(input)
}

/// Parses top level comment in the Ledger file format.
/// Notable difference with block_metadata is, this accepts multiple prefix.
pub fn top_comment<'i, I, E>(input: &mut I) -> ModalResult<syntax::TopLevelComment<'i>, E>
where
    I: Stream<Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "directive::top_comment",
        multiline_text(take_while(1.., is_comment_prefix)).map(syntax::TopLevelComment),
    )
    .parse_next(input)
}

/// Parses multi-line text with preceding prefix.
fn multiline_text<'a, I, E, F, O1>(prefix: F) -> impl Parser<I, Cow<'a, str>, E>
where
    I: Stream<Slice = &'a str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>,
    E: ParserError<I>,
    F: Parser<I, O1, E>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "directive::multiline_text",
        repeat(1.., delimited(prefix, till_line_ending, line_ending_or_eof))
            .fold(String::new, |mut ret, l| {
                ret.push_str(l);
                ret.push('\n');
                ret
            })
            .map(Cow::Owned),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::testing::expect_parse_ok;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use winnow::error::{ErrMode, InputError};

    #[test]
    fn account_declaration_without_details() {
        let input = "account Foo:Bar Baz";
        assert_eq!(
            expect_parse_ok(account_declaration, input),
            (
                "",
                syntax::AccountDeclaration {
                    name: "Foo:Bar Baz".into(),
                    details: vec![]
                }
            )
        );

        let input = "account Foo:Bar Baz\n2022";
        assert_eq!(
            expect_parse_ok(account_declaration, input),
            (
                "2022",
                syntax::AccountDeclaration {
                    name: "Foo:Bar Baz".into(),
                    details: vec![]
                }
            )
        );
    }

    #[test]
    fn account_declaration_with_details() {
        let input = indoc! {"
            account Foo:Bar
             ; comment1
               ; comment1-cont
               note note1
               alias alias1
               alias Alias 2:
               note note2
               note note2-cont

            2020"};
        assert_eq!(
            expect_parse_ok(account_declaration, input),
            (
                "\n2020",
                syntax::AccountDeclaration {
                    name: "Foo:Bar".into(),
                    details: vec![
                        syntax::AccountDetail::Comment(" comment1\n comment1-cont\n".into()),
                        syntax::AccountDetail::Note("note1\n".into()),
                        syntax::AccountDetail::Alias("alias1".into()),
                        syntax::AccountDetail::Alias("Alias 2:".into()),
                        syntax::AccountDetail::Note("note2\nnote2-cont\n".into()),
                    ],
                }
            )
        )
    }

    #[test]
    fn apply_tag_without_value() {
        let input = "apply tag foo   ";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                syntax::ApplyTag {
                    key: "foo".into(),
                    value: None,
                }
            )
        );

        let input = "apply  tag  test@1-2!#[]   \napply";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "apply",
                syntax::ApplyTag {
                    key: "test@1-2!#[]".into(),
                    value: None,
                }
            )
        );
    }
    #[test]
    fn apply_tag_with_value() {
        let input = "apply tag foo:bar\napply tag key";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "apply tag key",
                syntax::ApplyTag {
                    key: "foo".into(),
                    value: Some(syntax::MetadataValue::Text("bar".into())),
                }
            )
        );

        let input = "apply tag foo: bar  ";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                syntax::ApplyTag {
                    key: "foo".into(),
                    value: Some(syntax::MetadataValue::Text("bar".into())),
                }
            )
        );

        let input = "apply tag test@1-2!#[]  ::  [2022-3-4]  \n";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                syntax::ApplyTag {
                    key: "test@1-2!#[]".into(),
                    value: Some(syntax::MetadataValue::Expr("[2022-3-4]".into())),
                }
            )
        );
    }

    #[test]
    fn end_apply_tag_accepts_valid_inputs() {
        let input: &str = "end apply tag";
        assert_eq!(expect_parse_ok(end_apply_tag, input), ("", "end apply tag"));

        let input: &str = "end apply tag  \nfoo";
        assert_eq!(
            expect_parse_ok(end_apply_tag, input),
            ("foo", "end apply tag")
        );
    }

    #[test]
    fn end_apply_tag_rejects_unexpected() {
        let input: &str = "end apply tag   following";
        assert_eq!(
            end_apply_tag.parse_peek(input),
            Err(ErrMode::Backtrack(InputError::at("following")))
        );

        let input: &str = "end applytag";
        assert_eq!(
            end_apply_tag.parse_peek(input),
            Err(ErrMode::Backtrack(InputError::at("tag")))
        );

        let input: &str = "endapply tag";
        assert_eq!(
            end_apply_tag.parse_peek(input),
            Err(ErrMode::Backtrack(InputError::at("apply tag")))
        );
    }

    #[test]
    fn include_parses_normal_file() {
        assert_eq!(
            expect_parse_ok(include, "include foobar.ledger\n"),
            ("", syntax::IncludeFile("foobar.ledger".into()))
        );
    }

    #[test]
    fn include_trims_space_in_end() {
        assert_eq!(
            expect_parse_ok(include, "include foobar.ledger  \n"),
            ("", syntax::IncludeFile("foobar.ledger".into()))
        );
    }

    #[test]
    fn include_keeps_spaces_in_the_middle() {
        assert_eq!(
            expect_parse_ok(include, "include\t\t /path/to/foo bar.ledger  \n"),
            ("", syntax::IncludeFile("/path/to/foo bar.ledger".into()))
        );
    }

    #[test]
    fn top_comment_single_line() {
        assert_eq!(
            expect_parse_ok(top_comment, ";foo"),
            ("", syntax::TopLevelComment("foo\n".into()))
        );
        assert_eq!(
            expect_parse_ok(top_comment, ";foo\nbaz"),
            ("baz", syntax::TopLevelComment("foo\n".into()))
        );
    }

    #[test]
    fn top_comment_multi_lines() {
        assert_eq!(
            expect_parse_ok(top_comment, ";foo\n;bar"),
            ("", syntax::TopLevelComment("foo\nbar\n".into()))
        );
        assert_eq!(
            expect_parse_ok(top_comment, ";foo\n#bar\nbaz"),
            ("baz", syntax::TopLevelComment("foo\nbar\n".into()))
        );
    }
}
