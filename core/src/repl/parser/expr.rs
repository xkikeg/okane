//! Defines parsers related to value expression.

use crate::repl::{
    self, expr,
    parser::{
        combinator::{cond_else, has_peek},
        primitive,
    },
};

use nom::{
    character::complete::{char, one_of, space0},
    combinator::{fail, map, opt},
    error::{context, ContextError, FromExternalError, ParseError},
    sequence::{delimited, pair, terminated},
    IResult, InputLength, Parser,
};

/// Parses value expression.
pub fn value_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>,
{
    let (input, is_paren) = has_peek(char('('))(input)?;
    context("value-expr", cond_else(is_paren, paren, amount))(input)
}

fn paren<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>,
{
    map(
        delimited(pair(char('('), space0), add_expr, pair(space0, char(')'))),
        expr::ValueExpr::Paren,
    )(input)
}

fn add_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>,
{
    infixl(add_op, mul_expr)(input)
}

fn add_op<'a, E>(input: &'a str) -> IResult<&'a str, expr::BinaryOp, E>
where
    E: ParseError<&'a str>,
{
    map(delimited(space0, one_of("+-"), space0), |c| match c {
        '+' => expr::BinaryOp::Add,
        '-' => expr::BinaryOp::Sub,
        _ => unreachable!("add_op unexpected char"),
    })(input)
}

fn mul_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>
        + ContextError<&'a str>,
{
    infixl(mul_op, unary_expr)(input)
}

fn mul_op<'a, E>(input: &'a str) -> IResult<&'a str, expr::BinaryOp, E>
where
    E: ParseError<&'a str>,
{
    map(delimited(space0, one_of("*/"), space0), |c| match c {
        '*' => expr::BinaryOp::Mul,
        '/' => expr::BinaryOp::Div,
        _ => unreachable!("mul_op unexpected char"),
    })(input)
}

fn unary_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, rust_decimal::Error>
        + ContextError<&'a str>,
{
    let (input, negate) = opt(char('-'))(input)?;
    let (input, value) = value_expr(input)?;
    let value = expr::Expr::Value(Box::new(value));
    let expr = if negate.is_some() {
        expr::Expr::Unary(expr::UnaryOpExpr {
            op: expr::UnaryOp::Negate,
            expr: Box::new(value),
        })
    } else {
        value
    };
    Ok((input, expr))
}

/// Parses amount expression.
pub fn amount<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
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

/// Parses `x (op x)*` format, and feed the list into the given function.
/// This is similar to foldl, so it'll be evaluated as `f(f(...f(x, x), x), ... x)))`.
/// operand parser needs to be Copy so that it can be used twice.
fn infixl<'a, E, F, G>(
    mut operator: F,
    mut operand: G,
) -> impl FnMut(&'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
    F: Parser<&'a str, expr::BinaryOp, E>,
    G: Parser<&'a str, expr::Expr, E>,
{
    move |i: &'a str| {
        let (mut i, mut ret) = operand.parse(i)?;
        loop {
            match operator.parse(i) {
                Err(nom::Err::Error(_)) => return Ok((i, ret)),
                Err(x) => return Err(x),
                Ok((i1, op)) => {
                    if i1.input_len() == i.input_len() {
                        return context("infixl operator is empty", fail)(i);
                    }
                    i = i1;
                    let (i1, rhs) = operand.parse(i)?;
                    i = i1;
                    ret = expr::Expr::Binary(expr::BinaryOpExpr {
                        lhs: Box::new(ret),
                        op,
                        rhs: Box::new(rhs),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn value_expr_literal() {
        assert_eq!(
            expect_parse_ok(value_expr, "1000 JPY"),
            (
                "",
                expr::ValueExpr::Amount(repl::Amount {
                    value: dec!(1000),
                    commodity: "JPY".to_string()
                }),
            )
        );

        assert_eq!(
            expect_parse_ok(value_expr, "1,234,567.89 USD"),
            (
                "",
                expr::ValueExpr::Amount(repl::Amount {
                    value: dec!(1234567.89),
                    commodity: "USD".to_string()
                })
            )
        );
    }

    fn amount_expr<T: Into<Decimal>>(value: T, commodity: &'static str) -> expr::Expr {
        expr::Expr::Value(Box::new(expr::ValueExpr::Amount(repl::Amount {
            commodity: commodity.to_string(),
            value: value.into(),
        })))
    }

    #[test]
    fn value_expr_complex() {
        let input = "(0 - -(1.20 + 2.67) * 3.1 USD + 5 USD)";
        let want = expr::ValueExpr::Paren(expr::Expr::Binary(expr::BinaryOpExpr {
            lhs: Box::new(expr::Expr::Binary(expr::BinaryOpExpr {
                lhs: Box::new(amount_expr(dec!(0), "")),
                op: expr::BinaryOp::Sub,
                rhs: Box::new(expr::Expr::Binary(expr::BinaryOpExpr {
                    lhs: Box::new(expr::Expr::Unary(expr::UnaryOpExpr {
                        op: expr::UnaryOp::Negate,
                        expr: Box::new(expr::Expr::Value(Box::new(expr::ValueExpr::Paren(
                            expr::Expr::Binary(expr::BinaryOpExpr {
                                lhs: Box::new(amount_expr(dec!(1.20), "")),
                                op: expr::BinaryOp::Add,
                                rhs: Box::new(amount_expr(dec!(2.67), "")),
                            }),
                        )))),
                    })),
                    op: expr::BinaryOp::Mul,
                    rhs: Box::new(amount_expr(dec!(3.1), "USD")),
                })),
            })),
            op: expr::BinaryOp::Add,
            rhs: Box::new(amount_expr(5, "USD")),
        }));

        assert_eq!(expect_parse_ok(value_expr, input), ("", want))
    }
}
