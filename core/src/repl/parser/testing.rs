//! Utility only meant for the tests.

use winnow::{
    error::ContextError,
    stream::{AsBStr, AsChar, Stream, StreamIsPartial},
    token::take_while,
    Parser,
};

/// Run the given `parser` with the `input`, and returns a tuple of reamining input and output.
/// Panic when the parsing failed.
pub fn expect_parse_ok<I, O, F>(parser: F, input: I) -> (<I as Stream>::Slice, O)
where
    I: Stream + StreamIsPartial + Clone + AsBStr,
    <I as Stream>::Token: AsChar,
    F: Parser<I, O, ContextError>,
{
    match (parser, take_while(0.., |_x| true)).parse(input) {
        Ok((ret, remaining)) => (remaining, ret),
        Err(e) => panic!("failed to parse: {}", e),
    }
}
