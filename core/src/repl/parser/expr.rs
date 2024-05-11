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
    combinator::{alt, delimited, fail, opt, terminated, trace},
    error::{AddContext, FromExternalError, ParserError},
    token::one_of,
    PResult, Parser,
};

/// Parses value expression.
pub fn value_expr<'a, E, C>(input: &mut &'a str) -> PResult<expr::ValueExpr, E>
where
    E: ParserError<&'a str>
        + AddContext<&'a str, C>
        + FromExternalError<&'a str, pretty_decimal::Error>,
{
    let is_paren = has_peek(one_of('(')).parse_next(input)?;
    // TODO: Try to remove cond_else / has_peek because it can be removed with backtrack + cut_err.
    trace(
        "value_expr",
        cond_else(is_paren, paren_expr, amount.map(expr::ValueExpr::Amount)),
    )
    .parse_next(input)
}

fn paren_expr<'a, E, C>(input: &mut &'a str) -> PResult<expr::ValueExpr, E>
where
    E: ParserError<&'a str>
        + AddContext<&'a str, C>
        + FromExternalError<&'a str, pretty_decimal::Error>,
{
    delimited((one_of('('), space0), add_expr, (space0, one_of(')')))
        .map(expr::ValueExpr::Paren)
        .parse_next(input)
}

fn add_expr<'a, E, C>(input: &mut &'a str) -> PResult<expr::Expr, E>
where
    E: ParserError<&'a str>
        + AddContext<&'a str, C>
        + FromExternalError<&'a str, pretty_decimal::Error>,
{
    infixl(add_op, mul_expr).parse_next(input)
}

fn add_op<'a, E>(input: &mut &'a str) -> PResult<expr::BinaryOp, E>
where
    E: ParserError<&'a str>,
{
    delimited(
        space0,
        alt((
            one_of('+').value(expr::BinaryOp::Add),
            one_of('-').value(expr::BinaryOp::Sub),
        )),
        space0,
    )
    .parse_next(input)
}

fn mul_expr<'a, E, C>(input: &mut &'a str) -> PResult<expr::Expr, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>
        + AddContext<&'a str, C>,
{
    infixl(mul_op, unary_expr).parse_next(input)
}

fn mul_op<'a, E>(input: &mut &'a str) -> PResult<expr::BinaryOp, E>
where
    E: ParserError<&'a str>,
{
    delimited(
        space0,
        alt((
            one_of('*').value(expr::BinaryOp::Mul),
            one_of('/').value(expr::BinaryOp::Div),
        )),
        space0,
    )
    .parse_next(input)
}

fn unary_expr<'a, E, C>(input: &mut &'a str) -> PResult<expr::Expr, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>
        + AddContext<&'a str, C>,
{
    let negate = opt(one_of('-')).parse_next(input)?;
    let value = value_expr(input)?;
    let value = expr::Expr::Value(Box::new(value));
    let expr = if negate.is_some() {
        expr::Expr::Unary(expr::UnaryOpExpr {
            op: expr::UnaryOp::Negate,
            expr: Box::new(value),
        })
    } else {
        value
    };
    Ok(expr)
}

/// Parses amount expression.
pub fn amount<'a, E, C>(input: &mut &'a str) -> PResult<expr::Amount, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, pretty_decimal::Error>
        + AddContext<&'a str, C>,
{
    // Currently it only supports suffix commodity.
    // It should support prefix like $, € or ¥ prefix.
    let value = terminated(primitive::comma_decimal, space0).parse_next(input)?;
    let c = primitive::commodity(input)?;
    Ok(expr::Amount {
        value,
        commodity: c.to_string(),
    })
}

/// Parses `x (op x)*` format, and feed the list into the given function.
/// This is similar to foldl, so it'll be evaluated as `f(f(...f(x, x), x), ... x)))`.
/// operand parser needs to be Copy so that it can be used twice.
fn infixl<I, E, F, G, C>(mut operator: F, mut operand: G) -> impl Parser<I, expr::Expr, E>
where
    E: ParserError<I> + AddContext<I, C>,
    F: Parser<I, expr::BinaryOp, E>,
    G: Parser<I, expr::Expr, E>,
    I: winnow::stream::Stream + Clone,
{
    trace("infixl", move |i: &mut I| {
        let mut ret = operand.parse_next(i)?;
        loop {
            match operator.parse_peek(i.clone()) {
                Err(winnow::error::ErrMode::Backtrack(_)) => return Ok(ret),
                Err(x) => return Err(x),
                Ok((i1, op)) => {
                    if i1.eof_offset() == i.eof_offset() {
                        return fail.parse_next(i);
                    }
                    *i = i1;
                    let rhs = operand.parse_next(i)?;
                    ret = expr::Expr::Binary(expr::BinaryOpExpr {
                        lhs: Box::new(ret),
                        op,
                        rhs: Box::new(rhs),
                    });
                }
            }
        }
    })
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
