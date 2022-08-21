//! Defines parsers related to value expression.

use crate::repl::parser::{
    combinator::{cond_else, has_peek},
    primitive,
};
use crate::repl::{self, expr};

use nom::error::FromExternalError;
use nom::sequence::terminated;
use nom::{
    character::complete::char,
    error::{context, ParseError},
    IResult,
};
use nom::{character::complete::space0, combinator::fail, error::ContextError};

/// Parses value expression.
pub fn value_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>
        + ContextError<&'a str>,
{
    let (input, is_paren) = has_peek(char('('))(input)?;
    context("value-expr", cond_else(is_paren, paren, amount))(input)
}

fn paren<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParseError<&'a str>,
{
    fail(input)
}

/// Parses amount expression.
fn amount<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>
        + ContextError<&'a str>,
{
    // Currently it only supports suffix commodity.
    // It should support prefix like $, € or ¥ prefix.
    let (input, value) = terminated(primitive::comma_decimal, space0)(input)?;
    let (input, c) = primitive::commodity(input)?;
    Ok((
        input,
        repl::expr::ValueExpr::Amount(repl::Amount {
            value,
            commodity: c.to_string(),
        }),
    ))
}
