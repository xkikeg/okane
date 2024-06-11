//! Computes amount expressions.

use rust_decimal::Decimal;

use crate::{
    eval::{context, error::EvalError, types},
    repl::{self, expr},
};

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{AddAssign, Mul, MulAssign},
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Amount<'ctx> {
    values: HashMap<types::Commodity<'ctx>, Decimal>,
}

impl<'ctx> Amount<'ctx> {
    pub fn as_inline<'a>(&'a self) -> InlinePrintAmount<'a, 'ctx> {
        InlinePrintAmount(self)
    }

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

#[derive(Debug)]
pub struct InlinePrintAmount<'a, 'ctx>(&'a Amount<'ctx>);

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

#[derive(Debug, PartialEq, Eq)]
pub enum PartialAmount<'ctx> {
    Number(Decimal),
    Commodities(Amount<'ctx>),
}

impl<'ctx> PartialAmount<'ctx> {
    fn from_expr_amount(
        ctx: &mut context::EvalContext<'ctx>,
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

#[derive(Debug, Default)]
pub struct Balance<'ctx> {
    // TODO: Make it private.
    pub accounts: HashMap<types::Account<'ctx>, Amount<'ctx>>,
}

// TODO: Consider if this is ok to be private or needs to be pub.
pub(crate) trait Evaluable {
    fn eval<'ctx>(
        &self,
        ctx: &mut context::EvalContext<'ctx>,
    ) -> Result<PartialAmount<'ctx>, EvalError>;
}

impl Evaluable for expr::ValueExpr {
    fn eval<'ctx>(
        &self,
        ctx: &mut context::EvalContext<'ctx>,
    ) -> Result<PartialAmount<'ctx>, EvalError> {
        match self {
            expr::ValueExpr::Paren(x) => x.eval(ctx),
            expr::ValueExpr::Amount(x) => Ok(PartialAmount::from_expr_amount(ctx, x)),
        }
    }
}

impl Evaluable for expr::Expr {
    fn eval<'ctx>(
        &self,
        ctx: &mut context::EvalContext<'ctx>,
    ) -> Result<PartialAmount<'ctx>, EvalError> {
        match self {
            expr::Expr::Unary(e) => e.eval(ctx),
            expr::Expr::Binary(e) => e.eval(ctx),
            expr::Expr::Value(e) => e.eval(ctx),
        }
    }
}

impl Evaluable for expr::UnaryOpExpr {
    fn eval<'ctx>(
        &self,
        ctx: &mut context::EvalContext<'ctx>,
    ) -> Result<PartialAmount<'ctx>, EvalError> {
        match self.op {
            expr::UnaryOp::Negate => {
                let val = self.expr.eval(ctx)?;
                Ok(val.negate())
            }
        }
    }
}

impl Evaluable for expr::BinaryOpExpr {
    fn eval<'ctx>(
        &self,
        ctx: &mut context::EvalContext<'ctx>,
    ) -> Result<PartialAmount<'ctx>, EvalError> {
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

// impl<'ctx> Balance<'ctx> {
//     pub fn add_postings(&mut self, postings: &[repl::Posting]) -> Result<(), BalanceError> {
//         let txn_balance: Amount<'ctx> = Amount::default();
//         for posting in postings {

//             let balance = self
//                 .accounts
//                 .entry(&posting.account)
//                 .or_insert(Amount::default());
//             balance.add
//         }
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    use crate::repl::pretty_decimal::PrettyDecimal;

    use bumpalo::Bump;
    use context::EvalContext;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn eval_expr_simple() {
        let input = expr::ValueExpr::Amount(expr::Amount {
            value: PrettyDecimal::plain(dec!(100.12345)),
            commodity: "USD".to_string(),
        });
        let arena = Bump::new();
        let mut ctx = EvalContext::new(&arena);
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
        let (_, input) =
            repl::parser::testing::expect_parse_ok(repl::parser::expr::value_expr, input);
        let arena = Bump::new();
        let mut ctx = EvalContext::new(&arena);
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
        let (_, input) =
            repl::parser::testing::expect_parse_ok(repl::parser::expr::value_expr, input);
        let arena = Bump::new();
        let mut ctx = EvalContext::new(&arena);
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
