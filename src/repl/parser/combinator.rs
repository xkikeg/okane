//! Defines abstracted utility to work with other parsers.

use nom::{
    combinator::{map, opt, peek},
    error::ParseError,
    IResult, Parser,
};

/// Calls first parser if the condition is met, otherwise the second parser.
pub fn branch<I, O, E: ParseError<I>, F, G>(
    b: bool,
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: Parser<I, O, E>,
    G: Parser<I, O, E>,
{
    move |input: I| {
        if b {
            first.parse(input)
        } else {
            second.parse(input)
        }
    }
}

/// Returns true if given parser would succeed, without consuming the input.
pub fn has_peek<I, O, E: ParseError<I>, F>(f: F) -> impl FnMut(I) -> IResult<I, bool, E>
where
    F: Parser<I, O, E>,
    I: Clone,
{
    map(peek(opt(f)), |x| x.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use nom::bytes::complete::is_a;
    use pretty_assertions::assert_eq;

    #[test]
    fn branch_takes_first_given_true() {
        assert_eq!(
            expect_parse_ok(branch(true, is_a("true"), is_a("false")), "true"),
            ("", "true")
        );
    }

    #[test]
    fn branch_takes_second_given_false() {
        assert_eq!(
            expect_parse_ok(branch(false, is_a("true"), is_a("false")), "false"),
            ("", "false")
        );
    }

    #[test]
    fn has_peek_succeeds() {
        assert_eq!(
            expect_parse_ok(has_peek(is_a("abc")), "abcde"),
            ("abcde", true)
        );
        assert_eq!(
            expect_parse_ok(has_peek(is_a("0123")), "abcde"),
            ("abcde", false)
        );
    }
}
