//! Defines parsers related to value expression.

use crate::repl::{
    expr,
    parser::{
        combinator::{cond_else, has_peek},
        primitive,
    },
    pretty_decimal,
};

use winnow::{
    ascii::space0,
    bytes::one_of,
    combinator::{fail, opt},
    error::{AddContext, FromExternalError, ParserError},
    sequence::{delimited, terminated},
    IResult, Parser,
};

/// Parses value expression.
pub fn value_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParserError<&'a str>
        + AddContext<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>,
{
    let (input, is_paren) = has_peek(one_of('(')).parse_next(input)?;

    // TODO: Try to remove cond_else / has_peek because it can be removed with backtrack + cut_err.
    cond_else(is_paren, paren_expr, amount.map(expr::ValueExpr::Amount))
        .context("value-expr")
        .parse_next(input)
}

fn paren_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::ValueExpr, E>
where
    E: ParserError<&'a str>
        + AddContext<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>,
{
    delimited((one_of('('), space0), add_expr, (space0, one_of(')')))
        .map(expr::ValueExpr::Paren)
        .parse_next(input)
}

fn add_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParserError<&'a str>
        + AddContext<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>,
{
    infixl(add_op, mul_expr).parse_next(input)
}

fn add_op<'a, E>(input: &'a str) -> IResult<&'a str, expr::BinaryOp, E>
where
    E: ParserError<&'a str>,
{
    delimited(space0, one_of("+-"), space0)
        .map(|c| match c {
            '+' => expr::BinaryOp::Add,
            '-' => expr::BinaryOp::Sub,
            _ => unreachable!("add_op unexpected char"),
        })
        .parse_next(input)
}

fn mul_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>
        + AddContext<&'a str>,
{
    infixl(mul_op, unary_expr).parse_next(input)
}

fn mul_op<'a, E>(input: &'a str) -> IResult<&'a str, expr::BinaryOp, E>
where
    E: ParserError<&'a str>,
{
    delimited(space0, one_of("*/"), space0)
        .map(|c| match c {
            '*' => expr::BinaryOp::Mul,
            '/' => expr::BinaryOp::Div,
            _ => unreachable!("mul_op unexpected char"),
        })
        .parse_next(input)
}

fn unary_expr<'a, E>(input: &'a str) -> IResult<&'a str, expr::Expr, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>
        + AddContext<&'a str>,
{
    let (input, negate) = opt(one_of('-')).parse_next(input)?;
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
pub fn amount<'a, E>(input: &'a str) -> IResult<&'a str, expr::Amount, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>
        + AddContext<&'a str>,
{
    // Currently it only supports suffix commodity.
    // It should support prefix like $, € or ¥ prefix.
    let (input, value) = terminated(primitive::comma_decimal, space0).parse_next(input)?;
    let (input, c) = primitive::commodity(input)?;
    Ok((
        input,
        expr::Amount {
            value,
            commodity: c.to_string(),
        },
    ))
}

/// Parses `x (op x)*` format, and feed the list into the given function.
/// This is similar to foldl, so it'll be evaluated as `f(f(...f(x, x), x), ... x)))`.
/// operand parser needs to be Copy so that it can be used twice.
fn infixl<I, E, F, G>(mut operator: F, mut operand: G) -> impl Parser<I, expr::Expr, E>
where
    E: ParserError<I> + AddContext<I>,
    F: Parser<I, expr::BinaryOp, E>,
    G: Parser<I, expr::Expr, E>,
    I: winnow::stream::Stream,
{
    move |i: I| {
        let (mut i, mut ret) = operand.parse_next(i)?;
        loop {
            match operator.parse_next(i.clone()) {
                Err(winnow::error::ErrMode::Backtrack(_)) => return Ok((i, ret)),
                Err(x) => return Err(x),
                Ok((i1, op)) => {
                    if i1.eof_offset() == i.eof_offset() {
                        return fail.context("infixl operator is empty").parse_next(i);
                    }
                    i = i1;
                    let (i1, rhs) = operand.parse_next(i)?;
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

    use crate::repl::parser::testing::expect_parse_ok;
    use crate::repl::pretty_decimal::PrettyDecimal;

    use pretty_assertions::assert_eq;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn value_expr_literal() {
        assert_eq!(
            expect_parse_ok(value_expr, "1000 JPY"),
            (
                "",
                expr::ValueExpr::Amount(expr::Amount {
                    value: PrettyDecimal::plain(dec!(1000)),
                    commodity: "JPY".to_string()
                }),
            )
        );

        assert_eq!(
            expect_parse_ok(value_expr, "1,234,567.89 USD"),
            (
                "",
                expr::ValueExpr::Amount(expr::Amount {
                    value: PrettyDecimal::comma3dot(dec!(1234567.89)),
                    commodity: "USD".to_string()
                })
            )
        );
    }

    fn amount_expr<T: Into<Decimal>>(value: T, commodity: &'static str) -> expr::Expr {
        let v: Decimal = value.into();
        expr::Expr::Value(Box::new(expr::ValueExpr::Amount(expr::Amount {
            commodity: commodity.to_string(),
            value: PrettyDecimal::unformatted(v),
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
