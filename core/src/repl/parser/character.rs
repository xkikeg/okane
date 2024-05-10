//! Defines parser functions related to character input.

use crate::repl::parser;
use parser::combinator::{cond_else, has_peek};

use winnow::{
    branch::alt,
    bytes::{one_of, take_till1, take_while0},
    character::line_ending,
    combinator::eof,
    error::ParseError,
    sequence::delimited,
    IResult, Parser,
};

/// Semicolon or line ending.
pub fn line_ending_or_semi<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    let (input, is_semi) = has_peek(one_of(';')).parse_next(input)?;
    cond_else(is_semi, one_of(';').recognize(), line_ending).parse_next(input)
}

/// Parses non-zero string until line_ending or comma appears.
pub fn not_line_ending_or_semi<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&str, &str, E> {
    take_till1(";\r\n")(input)
}

/// Line ending or EOF.
pub fn line_ending_or_eof<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    alt((eof, line_ending))(input)
}

/// Parses unnested string in paren.
pub fn paren_str<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    paren(take_while0(|c| c != ')')).parse_next(input)
}

/// Parses given parser within the paren.
pub fn paren<I, O, E: ParseError<I>, F>(inner: F) -> impl Parser<I, O, E>
where
    F: Parser<I, O, E>,
    I: winnow::stream::Stream + winnow::stream::StreamIsPartial,
    <I as winnow::stream::Stream>::Token: winnow::stream::AsChar + Copy,
{
    delimited(one_of('('), inner, one_of(')'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use winnow::bytes::take_while1;

    #[test]
    fn line_ending_or_semi_accepts_valid_input() {
        assert_eq!(
            expect_parse_ok(line_ending_or_semi, ";remain"),
            ("remain", ";")
        );
        assert_eq!(
            expect_parse_ok(line_ending_or_semi, "\n;remain"),
            (";remain", "\n")
        );
        assert_eq!(
            expect_parse_ok(line_ending_or_semi, "\r\n;remain"),
            (";remain", "\r\n")
        );
    }

    #[test]
    fn paren_str_valid() {
        assert_eq!(
            expect_parse_ok(paren_str, "(this is a pen)"),
            ("", "this is a pen")
        )
    }

    #[test]
    fn paren_valid() {
        assert_eq!(
            expect_parse_ok(paren(paren(take_while1("abc"))), "((abcbca))"),
            ("", "abcbca")
        )
    }
}
