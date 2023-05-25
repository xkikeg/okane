use crate::repl;

use super::{
    character::line_ending_or_eof,
    metadata::{self},
};

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::{not_line_ending, space0, space1},
    combinator::{map, opt, recognize},
    error::{context, ContextError, ParseError},
    multi::{fold_many1, many0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};

/// Parses "account" directive.
pub fn account_declaration<'a, E>(input: &'a str) -> IResult<&'a str, repl::AccountDeclaration, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        pair(
            delimited(
                pair(tag("account"), space1),
                not_line_ending,
                line_ending_or_eof,
            ),
            // Note nesting many0 would cause parse failure at nom 7,
            // as many0 would fail if the sub-parser consumes empty input.
            // So make sure no branches in alt would emit zero input as success.
            many0(alt((
                map(
                    multiline_text(pair(space1, is_a(COMMENT_PREFIX))),
                    repl::AccountDetail::Comment,
                ),
                map(
                    multiline_text(tuple((space1, tag("note"), space1))),
                    repl::AccountDetail::Note,
                ),
                map(
                    delimited(
                        tuple((space1, tag("alias"), space1)),
                        not_line_ending,
                        line_ending_or_eof,
                    ),
                    |a| repl::AccountDetail::Alias(a.trim_end().to_string()),
                ),
            ))),
        ),
        |(name, details)| repl::AccountDeclaration {
            name: name.trim_end().to_string(),
            details,
        },
    )(input)
}

/// Parses "commodity" directive.
pub fn commodity_declaration<'a, E>(
    input: &'a str,
) -> IResult<&'a str, repl::CommodityDeclaration, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        pair(
            delimited(
                pair(tag("commodity"), space1),
                not_line_ending,
                line_ending_or_eof,
            ),
            // Note nesting many0 would cause parse failure at nom 7,
            // as many0 would fail if the sub-parser consumes empty input.
            // So make sure no branches in alt would emit zero input as success.
            many0(alt((
                map(
                    multiline_text(pair(space1, is_a(COMMENT_PREFIX))),
                    repl::CommodityDetail::Comment,
                ),
                map(
                    multiline_text(tuple((space1, tag("note"), space1))),
                    repl::CommodityDetail::Note,
                ),
                map(
                    delimited(
                        tuple((space1, tag("alias"), space1)),
                        not_line_ending,
                        line_ending_or_eof,
                    ),
                    |a| repl::CommodityDetail::Alias(a.trim_end().to_string()),
                ),
            ))),
        ),
        |(name, details)| repl::CommodityDeclaration {
            name: name.trim_end().to_string(),
            details,
        },
    )(input)
}

/// Parses "apply tag" directive.
pub fn apply_tag<'a, E>(input: &'a str) -> IResult<&'a str, repl::ApplyTag, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    // TODO: value needs to be supported.
    let (input, key) = preceded(
        tuple((tag("apply"), space1, tag("tag"), space1)),
        metadata::tag_key,
    )(input)?;
    let (input, value) =
        delimited(space0, opt(metadata::metadata_value), line_ending_or_eof)(input)?;
    Ok((
        input,
        repl::ApplyTag {
            key: key.to_string(),
            value,
        },
    ))
}

/// Parses "end apply tag" directive.
///
/// Note:
/// "end" directive has complicated semantics and may allow "end" or "end apply".
/// Also comment requires "end" directive.
/// In the meantime, only "end apply tag" is supported, however,
/// pretty sure it'd be needed to rename and extend this function.
pub fn end_apply_tag<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "end apply tag",
        terminated(
            recognize(tuple((
                tag("end"),
                space1,
                tag("apply"),
                space1,
                tag("tag"),
            ))),
            pair(space0, line_ending_or_eof),
        ),
    )(input)
}

/// Parses include directive.
/// Note given we'll always have UTF-8 input,
/// we're not using PathBuf but String for the path.
pub fn include<'a, E>(input: &'a str) -> IResult<&'a str, repl::IncludeFile, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "include directive",
        map(
            delimited(
                pair(tag("include"), space1),
                not_line_ending,
                line_ending_or_eof,
            ),
            |x| repl::IncludeFile(x.trim_end().to_string()),
        ),
    )(input)
}

static COMMENT_PREFIX: &str = ";#%|*";

/// Parses top level comment in the Ledger file format.
/// Notable difference with block_metadata is, this accepts multiple prefix.
pub fn top_comment<'a, E>(input: &'a str) -> IResult<&'a str, repl::TopLevelComment, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "top level comment",
        map(multiline_text(is_a(COMMENT_PREFIX)), repl::TopLevelComment),
    )(input)
}

/// Parses multi-line text with preceding prefix.
fn multiline_text<'a, E, F, O1>(prefix: F) -> impl FnMut(&'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str>,
    F: Parser<&'a str, O1, E>,
{
    fold_many1(
        delimited(prefix, not_line_ending, line_ending_or_eof),
        String::new,
        |mut ret, l| {
            ret.push_str(l);
            ret.push('\n');
            ret
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn account_declaration_without_details() {
        let input = "account Foo:Bar Baz";
        assert_eq!(
            expect_parse_ok(account_declaration, input),
            (
                "",
                repl::AccountDeclaration {
                    name: "Foo:Bar Baz".to_string(),
                    details: vec![]
                }
            )
        );

        let input = "account Foo:Bar Baz\n2022";
        assert_eq!(
            expect_parse_ok(account_declaration, input),
            (
                "2022",
                repl::AccountDeclaration {
                    name: "Foo:Bar Baz".to_string(),
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
                repl::AccountDeclaration {
                    name: "Foo:Bar".to_string(),
                    details: vec![
                        repl::AccountDetail::Comment(" comment1\n comment1-cont\n".to_string()),
                        repl::AccountDetail::Note("note1\n".to_string()),
                        repl::AccountDetail::Alias("alias1".to_string()),
                        repl::AccountDetail::Alias("Alias 2:".to_string()),
                        repl::AccountDetail::Note("note2\nnote2-cont\n".to_string()),
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
                repl::ApplyTag {
                    key: "foo".to_string(),
                    value: None,
                }
            )
        );

        let input = "apply  tag  test@1-2!#[]   \n";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                repl::ApplyTag {
                    key: "test@1-2!#[]".to_string(),
                    value: None,
                }
            )
        );
    }
    #[test]
    fn apply_tag_with_value() {
        let input = "apply tag foo:bar\n";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                repl::ApplyTag {
                    key: "foo".to_string(),
                    value: Some(repl::MetadataValue::Text("bar".to_string())),
                }
            )
        );

        let input = "apply tag foo: bar  ";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                repl::ApplyTag {
                    key: "foo".to_string(),
                    value: Some(repl::MetadataValue::Text("bar".to_string())),
                }
            )
        );

        let input = "apply tag test@1-2!#[]  ::  [2022-3-4]  \n";
        assert_eq!(
            expect_parse_ok(apply_tag, input),
            (
                "",
                repl::ApplyTag {
                    key: "test@1-2!#[]".to_string(),
                    value: Some(repl::MetadataValue::Expr("[2022-3-4]".to_string())),
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
        let end_apply_tag = end_apply_tag::<nom::error::Error<_>>;

        let input: &str = "end apply tag   following";
        end_apply_tag(input).expect_err("should fail");

        let input: &str = "end applytag";
        end_apply_tag(input).expect_err("should fail");

        let input: &str = "endapply tag";
        end_apply_tag(input).expect_err("should fail");
    }

    #[test]
    fn include_parses_normal_file() {
        assert_eq!(
            expect_parse_ok(include, "include foobar.ledger\n"),
            ("", repl::IncludeFile("foobar.ledger".to_string()))
        );
    }

    #[test]
    fn include_trims_space_in_end() {
        assert_eq!(
            expect_parse_ok(include, "include foobar.ledger  \n"),
            ("", repl::IncludeFile("foobar.ledger".to_string()))
        );
    }

    #[test]
    fn include_keeps_spaces_in_the_middle() {
        assert_eq!(
            expect_parse_ok(include, "include\t\t /path/to/foo bar.ledger  \n"),
            ("", repl::IncludeFile("/path/to/foo bar.ledger".to_string()))
        );
    }

    #[test]
    fn top_comment_single_line() {
        assert_eq!(
            expect_parse_ok(top_comment, ";foo"),
            ("", repl::TopLevelComment("foo\n".to_string()))
        );
        assert_eq!(
            expect_parse_ok(top_comment, ";foo\nbaz"),
            ("baz", repl::TopLevelComment("foo\n".to_string()))
        );
    }

    #[test]
    fn top_comment_multi_lines() {
        assert_eq!(
            expect_parse_ok(top_comment, ";foo\n;bar"),
            ("", repl::TopLevelComment("foo\nbar\n".to_string()))
        );
        assert_eq!(
            expect_parse_ok(top_comment, ";foo\n#bar\nbaz"),
            ("baz", repl::TopLevelComment("foo\nbar\n".to_string()))
        );
    }
}
