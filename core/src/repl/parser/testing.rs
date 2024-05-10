//! Utility only meant for the tests.

use winnow::{
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
    match parser.parse_next(input) {
        Ok(res) => res,
        Err(e) => match e {
            winnow::error::ErrMode::Incomplete(_) => {
                panic!("failed with incomplete: input: {}", input)
            }
            winnow::error::ErrMode::Backtrack(e) => panic!("error: {}", convert_error(input, e)),
            winnow::error::ErrMode::Cut(e) => panic!("failure: {}", convert_error(input, e)),
        },
    }
}
