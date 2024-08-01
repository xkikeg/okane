//! Defines parser functions related to character input.

use winnow::{
    ascii::line_ending,
    combinator::{alt, delimited, eof, trace},
    error::ParserError,
    stream::{AsChar, Compare, Stream, StreamIsPartial},
    token::{one_of, take_till},
    PResult, Parser,
};

/// Semicolon or line ending.
pub fn line_ending_or_semi<I, E>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial + Stream,
    I: Compare<&'static str>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I>,
{
    trace(
        "character::line_ending_or_semi",
        alt((one_of(';').take(), line_ending)),
    )
    .parse_next(input)
}

/// Parses non-zero string until line_ending or comma appears.
pub fn till_line_ending_or_semi<I, E>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar,
{
    trace(
        "character::till_line_ending_or_semi",
        take_till(1.., [';', '\r', '\n']),
    )
    .parse_next(input)
}

/// Line ending or EOF.
pub fn line_ending_or_eof<I, E>(input: &mut I) -> PResult<(), E>
where
    I: Stream + StreamIsPartial + winnow::stream::Compare<&'static str>,
    <I as Stream>::Token: AsChar,
    E: ParserError<I>,
{
    trace("character::line_ending_or_eof", alt((eof, line_ending)))
        .void()
        .parse_next(input)
}

/// Parses unnested string in paren.
pub fn paren_str<I, E>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    paren(take_till(0.., ')')).parse_next(input)
}

/// Parses given parser within the paren.
pub fn paren<I, O, E: ParserError<I>, F>(inner: F) -> impl Parser<I, O, E>
where
    F: Parser<I, O, E>,
    I: winnow::stream::Stream + StreamIsPartial,
    <I as winnow::stream::Stream>::Token: winnow::stream::AsChar + Clone,
{
    trace(
        "character::paren",
        delimited(one_of('('), inner, one_of(')')),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use winnow::token::take_while;

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
            expect_parse_ok(paren(paren(take_while(1.., 'a'..='c'))), "((abcbca))"),
            ("", "abcbca")
        )
    }
}
