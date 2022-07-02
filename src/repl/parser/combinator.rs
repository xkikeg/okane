//! Defines abstracted utility to work with other parsers.

use nom::{
    combinator::{map, opt, peek},
    error::ParseError,
    IResult, Parser,
};

/// Calls first parser if the condition is met, otherwise the second parser.
pub fn cond_else<I, O, E: ParseError<I>, F, G>(
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
