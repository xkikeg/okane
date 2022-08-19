//! Utility only meant for the tests.

use nom::{
    error::{convert_error, VerboseError},
    Parser,
};

/// Run the given `parser` with the `input`, and returns a tuple of reamining input and output.
/// Panic when the parsing failed.
pub fn expect_parse_ok<I, O, F>(mut parser: F, input: I) -> (I, O)
where
    I: std::ops::Deref<Target = str> + std::fmt::Display + Copy,
    F: Parser<I, O, VerboseError<I>>,
{
    match parser.parse(input) {
        Ok(res) => res,
        Err(e) => match e {
            nom::Err::Incomplete(_) => panic!("failed with incomplete: input: {}", input),
            nom::Err::Error(e) => panic!("error: {}", convert_error(input, e)),
            nom::Err::Failure(e) => panic!("error: {}", convert_error(input, e)),
        },
    }
}
