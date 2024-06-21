//! Evaluation logics of expr.

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{AddAssign, Mul, MulAssign},
};

use rust_decimal::Decimal;

use super::{context::ReportContext, intern::Commodity};
use crate::repl::expr;

/// Returns error related to evaluation.
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("operator can't be applied to unmatched types")]
    UnmatchingOperation,
    #[error("cannot divide by zero")]
    DivideByZero,
    #[error("overflow happened")]
    NumberOverflow,
    #[error("expected 0 or amount with commodity: got {0}")]
    CommodityAmountRequired(String),
}

/// Amount with multiple commodities, or simple zero.
// TODO: Rename it to ValueAmount.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Amount<'ctx> {
    // if values.len == zero, then it'll be completely zero.
    // TODO: Consider optimizing for small number of commodities,
    // as most of the case it needs to be just a few elements.
    values: HashMap<Commodity<'ctx>, Decimal>,
}

impl<'ctx> Amount<'ctx> {
    /// Returns `InlinePrintAmount`, which is good for debugging display.
    pub fn as_inline_display<'a>(&'a self) -> impl Display + 'a {
        InlinePrintAmount(self)
    }

    /// Returns `true` if this is 'non-commoditized zero', which is used to assert
    /// the account balance is completely zero.
    pub fn is_absolute_zero(&self) -> bool {
        self.values.iter().all(|(_, v)| v.is_zero())
    }

    /// Returns `true` if this is zero, including zero commodities.
    pub fn is_zero(&self) -> bool {
        self.values.iter().all(|(_, v)| v.is_zero())
    }

    pub fn negate(mut self) -> Self {
        for (_, v) in self.values.iter_mut() {
            v.set_sign_positive(!v.is_sign_positive())
        }
        self
    }

    pub fn check_div(mut self, rhs: Decimal) -> Result<Self, EvalError> {
        if rhs.is_zero() {
            return Err(EvalError::DivideByZero);
        }
        for (_, v) in self.values.iter_mut() {
            *v = v.checked_div(rhs).ok_or(EvalError::NumberOverflow)?;
        }
        Ok(self)
    }
}

impl<'ctx> TryFrom<PartialAmount<'ctx>> for Amount<'ctx> {
    type Error = EvalError;

    fn try_from(value: PartialAmount<'ctx>) -> Result<Self, Self::Error> {
        match value {
            PartialAmount::Commodities(x) => Ok(x),
            PartialAmount::Number(x) if x.is_zero() => Ok(Self::default()),
            _ => Err(EvalError::CommodityAmountRequired(format!("{}", value))),
        }
    }
}

#[derive(Debug)]
struct InlinePrintAmount<'a, 'ctx>(&'a Amount<'ctx>);

impl<'a, 'ctx> Display for InlinePrintAmount<'a, 'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vs = &self.0.values;
        match vs.len() {
            0 | 1 => match vs.iter().next() {
                Some((c, v)) => write!(f, "{} {}", v, c.as_str()),
                None => write!(f, "0"),
            },
            _ => {
                write!(f, "(")?;
                for (i, (c, v)) in vs.iter().enumerate() {
                    if i != 0 {
                        write!(f, " + ")?;
                    }
                    write!(f, "{} {}", v, c.as_str())?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<'ctx> AddAssign for Amount<'ctx> {
    fn add_assign(&mut self, rhs: Self) {
        for (c, v2) in &rhs.values {
            let mut v1 = self.values.entry(*c).or_insert(Decimal::ZERO);
            v1 += v2;
            if v1.is_zero() {
                self.values.remove(c);
            }
        }
    }
}

impl<'ctx> Mul<Decimal> for Amount<'ctx> {
    type Output = Self;

    fn mul(mut self, rhs: Decimal) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'ctx> MulAssign<Decimal> for Amount<'ctx> {
    fn mul_assign(&mut self, rhs: Decimal) {
        for (_, mut v) in self.values.iter_mut() {
            v *= rhs;
        }
    }
}

/// Amount as the expression evaluation results.
// TODO: Rename it to ExprValue.
#[derive(Debug, PartialEq, Eq)]
pub enum PartialAmount<'ctx> {
    Number(Decimal),
    Commodities(Amount<'ctx>),
}

impl<'ctx> PartialAmount<'ctx> {
    fn from_expr_amount(
        ctx: &mut ReportContext<'ctx>,
        amount: &expr::Amount,
    ) -> PartialAmount<'ctx> {
        if amount.commodity.is_empty() {
            return Self::Number(amount.value.value);
        }
        let mut value = Amount {
            values: HashMap::new(),
        };
        let commodity = ctx.commodities.intern(&amount.commodity);
        value.values.insert(commodity, amount.value.value);
        PartialAmount::Commodities(value)
    }

    fn is_zero(&self) -> bool {
        match self {
            PartialAmount::Number(x) => x.is_zero(),
            PartialAmount::Commodities(y) => y.is_zero(),
        }
    }

    fn negate(self) -> Self {
        match self {
            PartialAmount::Number(x) => PartialAmount::Number(-x),
            PartialAmount::Commodities(x) => PartialAmount::Commodities(x.negate()),
        }
    }

    fn check_add(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (PartialAmount::Number(l), PartialAmount::Number(r)) => {
                Ok(PartialAmount::Number(l + r))
            }
            (PartialAmount::Commodities(mut l), PartialAmount::Commodities(r)) => {
                l += r;
                Ok(PartialAmount::Commodities(l))
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }

    fn check_sub(self, rhs: Self) -> Result<Self, EvalError> {
        self.check_add(rhs.negate())
    }

    fn check_mul(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (PartialAmount::Number(x), PartialAmount::Number(y)) => {
                Ok(PartialAmount::Number(x * y))
            }
            (PartialAmount::Commodities(x), PartialAmount::Number(y)) => {
                Ok(PartialAmount::Commodities(x * y))
            }
            (PartialAmount::Number(x), PartialAmount::Commodities(y)) => {
                Ok(PartialAmount::Commodities(y * x))
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }

    fn check_div(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (_, rhs) if rhs.is_zero() => Err(EvalError::DivideByZero),
            (PartialAmount::Number(x), PartialAmount::Number(y)) => {
                Ok(PartialAmount::Number(x / y))
            }
            (PartialAmount::Commodities(x), PartialAmount::Number(y)) => {
                x.check_div(y).map(PartialAmount::Commodities)
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }
}

impl<'ctx> Display for PartialAmount<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartialAmount::Number(x) => x.fmt(f),
            PartialAmount::Commodities(x) => x.as_inline_display().fmt(f),
        }
    }
}

// TODO: Consider if this is ok to be private or needs to be pub.
pub(crate) trait Evaluable {
    fn eval<'ctx>(&self, ctx: &mut ReportContext<'ctx>) -> Result<PartialAmount<'ctx>, EvalError>;
}

impl<'a> Evaluable for expr::ValueExpr<'a> {
    fn eval<'ctx>(&self, ctx: &mut ReportContext<'ctx>) -> Result<PartialAmount<'ctx>, EvalError> {
        match self {
            expr::ValueExpr::Paren(x) => x.eval(ctx),
            expr::ValueExpr::Amount(x) => Ok(PartialAmount::from_expr_amount(ctx, x)),
        }
    }
}

impl<'a> Evaluable for expr::Expr<'a> {
    fn eval<'ctx>(&self, ctx: &mut ReportContext<'ctx>) -> Result<PartialAmount<'ctx>, EvalError> {
        match self {
            expr::Expr::Unary(e) => e.eval(ctx),
            expr::Expr::Binary(e) => e.eval(ctx),
            expr::Expr::Value(e) => e.eval(ctx),
        }
    }
}

impl<'a> Evaluable for expr::UnaryOpExpr<'a> {
    fn eval<'ctx>(&self, ctx: &mut ReportContext<'ctx>) -> Result<PartialAmount<'ctx>, EvalError> {
        match self.op {
            expr::UnaryOp::Negate => {
                let val = self.expr.eval(ctx)?;
                Ok(val.negate())
            }
        }
    }
}

impl<'a> Evaluable for expr::BinaryOpExpr<'a> {
    fn eval<'ctx>(&self, ctx: &mut ReportContext<'ctx>) -> Result<PartialAmount<'ctx>, EvalError> {
        let lhs = self.lhs.eval(ctx)?;
        let rhs = self.rhs.eval(ctx)?;
        match self.op {
            expr::BinaryOp::Add => lhs.check_add(rhs),
            expr::BinaryOp::Sub => lhs.check_sub(rhs),
            expr::BinaryOp::Mul => lhs.check_mul(rhs),
            expr::BinaryOp::Div => lhs.check_div(rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::repl::pretty_decimal::PrettyDecimal;

    #[test]
    fn eval_expr_simple() {
        let input = expr::ValueExpr::Amount(expr::Amount {
            value: PrettyDecimal::plain(dec!(100.12345)),
            commodity: "USD".into(),
        });
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let got = input.eval(&mut ctx).unwrap();
        assert_eq!(
            PartialAmount::Commodities(Amount {
                values: hashmap! {
                    ctx.commodities.intern("USD") => dec!(100.12345),
                }
            }),
            got
        );
    }

    #[test]
    fn eval_expr_add_negate() {
        let input = "(100 USD + 300 EUR + (-100 USD + 20,000 JPY))";
        let input: expr::ValueExpr<'static> = input.try_into().expect("must succeed to parse");
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let got = input.eval(&mut ctx).unwrap();
        assert_eq!(
            PartialAmount::Commodities(Amount {
                values: hashmap! {
                    ctx.commodities.intern("EUR") => dec!(300),
                    ctx.commodities.intern("JPY") => dec!(20000),
                }
            }),
            got
        );
    }

    #[test]
    fn eval_expr_complex() {
        let input = "((100 USD + 200 EUR) * 2 - 100 USD / 5)";
        let input: expr::ValueExpr = input.try_into().expect("must not fail to parse");
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let got = input.eval(&mut ctx).unwrap();
        assert_eq!(
            PartialAmount::Commodities(Amount {
                values: hashmap! {
                    ctx.commodities.intern("USD") => dec!(180),
                    ctx.commodities.intern("EUR") => dec!(400),
                }
            }),
            got
        );
    }
}
