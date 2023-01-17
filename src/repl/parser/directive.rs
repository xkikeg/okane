use crate::repl;

use super::{character::line_ending_or_eof, metadata};

use nom::{
    bytes::complete::{is_a, tag},
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{map, recognize},
    error::{context, ContextError, ParseError},
    multi::fold_many1,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

/// Parses "apply tag" directive.
pub fn apply_tag<'a, E>(input: &'a str) -> IResult<&'a str, repl::ApplyTag, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    // TODO: value needs to be supported.
    let (input, tag) = preceded(
        tuple((tag("apply"), space1, tag("tag"), space1)),
        metadata::tag_key,
    )(input)?;
    let (input, _) = line_ending_or_eof(input)?;
    Ok((
        input,
        repl::ApplyTag {
            key: tag.to_string(),
            value: None,
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

/// Parses top level comment in the Ledger file format.
/// Notable difference with block_metadata is, this accepts multiple prefix.
pub fn top_comment<'a, E>(input: &'a str) -> IResult<&'a str, repl::TopLevelComment, E>
where
    E: ParseError<&'a str>,
{
    map(
        fold_many1(
            delimited(is_a(";#%|*"), not_line_ending, line_ending),
            String::new,
            |mut ret, l| {
                ret.push_str(l);
                ret.push('\n');
                ret
            },
        ),
        repl::TopLevelComment,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;

    #[test]
    fn apply_tag_without_value() {
        let input = "apply tag foo";
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

        let input = "apply tag test@1-2!#[]\n";
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
}
