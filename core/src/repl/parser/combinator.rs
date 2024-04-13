//! Defines abstracted utility to work with other parsers.

use nom::{
    combinator::{map, opt, peek},
    error::ParseError,
    IResult, InputLength, Parser,
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

/// Applies the parser `f` while it can peek with the parser `g`.
/// Returns a `Vec` containing the result of `f`.
/// 
/// Strictly speaking, this has the same output with `many0` as long as the input is well-formed.
/// However, it'll make the debug easier as `f` failure is immediately caught as long as `g` is met.
pub fn many0_while<I, O, P, E, F, G>(mut f: F, mut g: G) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: Parser<I, P, E>,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut ret = Vec::new();
        loop {
            let len = i.input_len();
            match g.parse(i.clone()) {
                Ok((_, _)) => (),
                Err(nom::Err::Error(_)) => return Ok((i, ret)),
                Err(e) => return Err(e),
            };
            let (i1, o) = f.parse(i)?;
            ret.push(o);
            if i1.input_len() == len {
                return Err(nom::Err::Error(E::from_error_kind(
                    i1,
                    nom::error::ErrorKind::ManyTill,
                )));
            }
            i = i1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use nom::{
        bytes::complete::is_a,
        character::complete::{anychar, char, space0, space1},
        sequence::preceded,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn cond_else_takes_first_given_true() {
        assert_eq!(
            expect_parse_ok(cond_else(true, is_a("true"), is_a("false")), "true"),
            ("", "true")
        );
    }

    #[test]
    fn cond_else_takes_second_given_false() {
        assert_eq!(
            expect_parse_ok(cond_else(false, is_a("true"), is_a("false")), "false"),
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

    #[test]
    fn many_while_zero() {
        assert_eq!(
            expect_parse_ok(many0_while(preceded(space1, is_a("abc")), char(' ')), "abc"),
            ("abc", vec![])
        )
    }

    #[test]
    fn many_while_some_items() {
        assert_eq!(
            expect_parse_ok(
                many0_while(preceded(space1, is_a("abc")), char(' ')),
                " abc   abc abc123"
            ),
            ("123", vec!["abc", "abc", "abc"])
        )
    }

    #[test]
    fn many_while_empty_item_error() {
        let _: nom::Err<nom::error::Error<&'static str>> =
            many0_while(space0, anychar)("abc").unwrap_err();
    }
}
