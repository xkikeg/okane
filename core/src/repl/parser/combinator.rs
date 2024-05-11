//! Defines abstracted utility to work with other parsers.

use winnow::{
    combinator::{opt, peek},
    error::ParseError,
    Parser,
};

/// Calls first parser if the condition is met, otherwise the second parser.
pub fn cond_else<I, O, E: ParseError<I>, F, G>(
    b: bool,
    mut first: F,
    mut second: G,
) -> impl Parser<I, O, E>
where
    F: Parser<I, O, E>,
    G: Parser<I, O, E>,
{
    move |input: I| {
        if b {
            first.parse_next(input)
        } else {
            second.parse_next(input)
        }
    }
}

/// Returns true if given parser would succeed, without consuming the input.
pub fn has_peek<I, O, E: ParseError<I>, F>(f: F) -> impl Parser<I, bool, E>
where
    F: Parser<I, O, E>,
    I: winnow::stream::Stream + Clone,
{
    peek(opt(f)).map(|x| x.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use winnow::{bytes::tag, token::take_while};

    #[test]
    fn cond_else_takes_first_given_true() {
        assert_eq!(
            expect_parse_ok(cond_else(true, tag("true"), tag("false")), "true"),
            ("", "true")
        );
    }

    #[test]
    fn cond_else_takes_second_given_false() {
        assert_eq!(
            expect_parse_ok(cond_else(false, tag("true"), tag("false")), "false"),
            ("", "false")
        );
    }

    #[test]
    fn has_peek_succeeds() {
        assert_eq!(
            expect_parse_ok(has_peek(take_while(1.., "abc")), "abcde"),
            ("abcde", true)
        );
        assert_eq!(
            expect_parse_ok(has_peek(take_while(1.., "0123")), "abcde"),
            ("abcde", false)
        );
    }
}
