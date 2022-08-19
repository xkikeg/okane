//! Defines parser functions related to character input.

use crate::repl::parser;
use parser::combinator::{branch, has_peek};

use nom::{
    branch::alt,
    bytes::complete::{is_not, take_while},
    character::complete::{char, line_ending},
    combinator::{eof, recognize},
    error::ParseError,
    sequence::delimited,
    IResult, Parser,
};

/// Semicolon or line ending.
pub fn line_ending_or_semi<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    let (input, is_semi) = has_peek(char(';'))(input)?;
    branch(is_semi, recognize(char(';')), line_ending)(input)
}

/// Parses non-zero string until line_ending or comma appears.
pub fn not_line_ending_or_semi<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&str, &str, E> {
    is_not(";\r\n")(input)
}

/// Line ending or EOF.
pub fn line_ending_or_eof<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    alt((eof, line_ending))(input)
}

/// Parses unnested string in paren.
pub fn paren_str<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    paren(take_while(|c| c != ')'))(input)
}

/// Parses given parser within the paren.
pub fn paren<I, O, E: ParseError<I>, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: Parser<I, O, E>,
    I: nom::Slice<core::ops::RangeFrom<usize>> + nom::InputIter,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    delimited(char('('), inner, char(')'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use nom::bytes::complete::is_a;
    use pretty_assertions::assert_eq;

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
            expect_parse_ok(paren(paren(is_a("abc"))), "((abcbca))"),
            ("", "abcbca")
        )
    }
}
