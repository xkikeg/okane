//! Defines parsers related to value expression.

use winnow::{
    ascii::space0,
    combinator::{
        alt, delimited, dispatch, opt, peek, permutation, preceded, separated_foldl1, terminated,
        trace,
    },
    error::{FromExternalError, ParserError},
    stream::{AsChar, Stream, StreamIsPartial},
    token::{any, one_of},
    Parser,
};

use crate::syntax::expr;

use super::{adaptor::ParseOptions, character::paren, error, primitive};

/// Parses value expression.
pub fn value_expr<'i, I, E>(input: &mut I) -> winnow::Result<expr::ValueExpr<'i>, E>
where
    I: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Clone,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "expr::value_expr",
        dispatch! {peek(any);
            '(' => paren_expr,
            _ => amount.map(expr::ValueExpr::Amount),
        },
    )
    .parse_next(input)
}

impl<'i> TryFrom<&'i str> for expr::ValueExpr<'i> {
    type Error = error::ParseError;

    fn try_from(value: &'i str) -> Result<Self, Self::Error> {
        ParseOptions::default()
            .parse_single(value_expr, value)
            .map(|x| x.1)
    }
}

fn paren_expr<'i, I, E>(input: &mut I) -> winnow::Result<expr::ValueExpr<'i>, E>
where
    I: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Clone,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "expr::paren_expr",
        paren(delimited(space0, add_expr, space0)).map(expr::ValueExpr::Paren),
    )
    .parse_next(input)
}

fn add_expr<'i, I, E>(input: &mut I) -> winnow::Result<expr::Expr<'i>, E>
where
    I: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Clone,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace("expr::add_expr", infixl(add_op, mul_expr)).parse_next(input)
}

fn add_op<I, E>(input: &mut I) -> winnow::Result<expr::BinaryOp, E>
where
    I: Stream + StreamIsPartial + Clone,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "expr::add_op",
        alt((
            one_of('+').value(expr::BinaryOp::Add),
            one_of('-').value(expr::BinaryOp::Sub),
        )),
    )
    .parse_next(input)
}

fn mul_expr<'i, I, E>(input: &mut I) -> winnow::Result<expr::Expr<'i>, E>
where
    I: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Clone,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace("expr::mul_expr", infixl(mul_op, unary_expr)).parse_next(input)
}

fn mul_op<I, E>(input: &mut I) -> winnow::Result<expr::BinaryOp, E>
where
    I: Stream + StreamIsPartial + Clone,
    E: ParserError<I>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "expr::mul_op",
        alt((
            one_of('*').value(expr::BinaryOp::Mul),
            one_of('/').value(expr::BinaryOp::Div),
        )),
    )
    .parse_next(input)
}

fn unary_expr<'i, I, E>(input: &mut I) -> winnow::Result<expr::Expr<'i>, E>
where
    I: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Clone,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "expr::unary_expr",
        dispatch! {peek(any);
            '-' => negate_expr,
            _ => value_expr.map(|ve| expr::Expr::Value(Box::new(ve))),
        },
    )
    .parse_next(input)
}

/// Parses amount expression, possibly with negative op.
// Only used for [`Amount`] `TryFrom<&'str>` implementation.
fn unary_amount<'i, I, E>(input: &mut I) -> winnow::Result<expr::Amount<'i>, E>
where
    I: Stream<Slice = &'i str> + StreamIsPartial,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    // This supports prefix commodity.
    trace(
        "expr::unary_amount",
        (
            opt(one_of('-')),
            permutation((
                terminated(primitive::pretty_decimal, space0),
                terminated(primitive::commodity, space0),
            )),
        )
            .map(|(negate, (mut value, c)): (_, (_, &str))| {
                if negate.is_some() {
                    value
                        .value
                        .set_sign_positive(!value.value.is_sign_positive());
                }
                expr::Amount {
                    value,
                    commodity: c.into(),
                }
            }),
    )
    .parse_next(input)
}

impl<'i> TryFrom<&'i str> for expr::Amount<'i> {
    type Error = error::ParseError;

    fn try_from(value: &'i str) -> Result<Self, Self::Error> {
        ParseOptions::default()
            .parse_single(unary_amount, value)
            .map(|x| x.1)
    }
}

fn negate_expr<'i, I, E>(input: &mut I) -> winnow::Result<expr::Expr<'i>, E>
where
    I: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Clone,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "expr::negate_expr",
        preceded(one_of('-'), value_expr).map(|ve| {
            expr::Expr::Unary(expr::UnaryOpExpr {
                op: expr::UnaryOp::Negate,
                expr: Box::new(expr::Expr::Value(Box::new(ve))),
            })
        }),
    )
    .parse_next(input)
}

/// Parses amount expression.
pub fn amount<'i, I, E>(input: &mut I) -> winnow::Result<expr::Amount<'i>, E>
where
    I: Stream<Slice = &'i str> + StreamIsPartial,
    E: ParserError<I> + FromExternalError<I, pretty_decimal::ParseError>,
    <I as Stream>::Token: AsChar + Clone,
{
    // Currently it only supports suffix commodity,
    // and there is no plan to support prefix commodities.
    trace(
        "expr::amount",
        (
            terminated(primitive::pretty_decimal, space0),
            primitive::commodity,
        )
            .map(|(value, c): (_, &str)| expr::Amount {
                value,
                commodity: c.into(),
            }),
    )
    .parse_next(input)
}

/// Parses `x (op x)*` format, and feed the list into the given function.
/// This is similar to foldl, so it'll be evaluated as `f(f(...f(x, x), x), ... x)))`.
/// operand parser needs to be Copy so that it can be used twice.
fn infixl<'i, I, E, F, G>(operator: F, operand: G) -> impl Parser<I, expr::Expr<'i>, E>
where
    I: Stream + StreamIsPartial + Clone,
    E: ParserError<I>,
    F: Parser<I, expr::BinaryOp, E>,
    G: Parser<I, expr::Expr<'i>, E>,
    I: Stream + StreamIsPartial + Clone,
    <I as Stream>::Token: AsChar,
{
    trace(
        "infixl",
        separated_foldl1(
            operand,
            delimited(space0, operator, space0),
            |lhs, op, rhs| {
                expr::Expr::Binary(expr::BinaryOpExpr {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                })
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parse::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use pretty_decimal::PrettyDecimal;
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
                    commodity: "JPY".into()
                }),
            )
        );

        assert_eq!(
            expect_parse_ok(value_expr, "1,234,567.89 USD"),
            (
                "",
                expr::ValueExpr::Amount(expr::Amount {
                    value: PrettyDecimal::comma3dot(dec!(1234567.89)),
                    commodity: "USD".into()
                })
            )
        );
    }

    fn amount_expr<T: Into<Decimal>>(value: T, commodity: &'static str) -> expr::Expr<'static> {
        let v: Decimal = value.into();
        expr::Expr::Value(Box::new(expr::ValueExpr::Amount(expr::Amount {
            commodity: commodity.into(),
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
