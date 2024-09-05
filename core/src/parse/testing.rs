//! Utility only meant for the tests.

use winnow::{error::ContextError, stream::Stream, token::take_while, Located, Parser};

/// Run the given `parser` with the `input`, and returns a tuple of reamining input and output.
/// Panic when the parsing failed.
pub fn expect_parse_ok<'i, O, F>(
    parser: F,
    input: &'i str,
) -> (<Located<&'i str> as Stream>::Slice, O)
where
    F: Parser<Located<&'i str>, O, ContextError>,
{
    match (parser, take_while(0.., |_x| true)).parse(Located::new(input)) {
        Ok((ret, remaining)) => (remaining, ret),
        Err(e) => panic!("failed to parse: \n{}", e),
    }
}
