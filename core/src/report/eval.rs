//! Evaluation logics of expressions.
//!
//! This module provides several types.
//! * [`Amount`] to represent general amount, which is commodity associted decimal amount.
//! * [`SingleAmount`] to represent an amount with only one commodity.
//! * [`Evaluated`] which is the result of syntax expression eval.

mod amount;
mod error;
mod evaluated;
mod posting_amount;
mod single_amount;

pub use amount::Amount;
pub use error::{EvalError, OwnedEvalError};
pub use evaluated::Evaluated;
pub(super) use posting_amount::PostingAmount;
pub use single_amount::SingleAmount;

use std::marker::PhantomData;

use super::context::ReportContext;
use crate::syntax::expr::{self, ExprVisitor, Visitable};

/// Provides syntax tree evaluation, on top of [`EvalExpr`].
pub(crate) trait Evaluable<'i>: Visitable<'i> {
    /// Evaluate the self with mutable `ctx`, which allows unknown commodities in the expressions to be registered.
    fn eval_mut<'ctx>(
        &self,
        ctx: &mut ReportContext<'ctx>,
    ) -> Result<Evaluated<'ctx>, EvalError<'ctx>> {
        self.accept(&mut EvalExpr::new(RegisterCommodity(ctx)))
    }

    /// Evaluate the self with immutable `ctx`, which raises error on unknown commodities.
    fn eval<'ctx>(&self, ctx: &ReportContext<'ctx>) -> Result<Evaluated<'ctx>, EvalError<'ctx>> {
        self.accept(&mut EvalExpr::new(ResolveCommodity(ctx)))
    }
}

impl<'i, T: Visitable<'i> + ?Sized> Evaluable<'i> for T {}

/// [`ExprVisitor`] evaluating an expression into [`Evaluated`].
///
/// Handling of the leaf is delegated to `A`, which decides whether an unknown commodity gets
/// registered ([`Evaluable::eval_mut`]) or rejected ([`Evaluable::eval`]).
struct EvalExpr<'ctx, A>(A, PhantomData<ReportContext<'ctx>>);

impl<'ctx, A: EvalAmount<'ctx>> EvalExpr<'ctx, A> {
    fn new(eval_amount: A) -> Self {
        EvalExpr(eval_amount, PhantomData)
    }
}

impl<'i, 'ctx, A: EvalAmount<'ctx>> ExprVisitor<'i> for EvalExpr<'ctx, A> {
    type Output = Evaluated<'ctx>;
    type Error = EvalError<'ctx>;

    fn visit_amount(&mut self, amount: &expr::Amount<'i>) -> Result<Self::Output, Self::Error> {
        self.0.eval_amount(amount)
    }

    fn visit_unary(&mut self, expr: &expr::UnaryOpExpr<'i>) -> Result<Self::Output, Self::Error> {
        match expr.op {
            expr::UnaryOp::Negate => {
                let val = self.visit_expr(&expr.expr)?;
                Ok(val.negate())
            }
        }
    }

    fn visit_binary(&mut self, expr: &expr::BinaryOpExpr<'i>) -> Result<Self::Output, Self::Error> {
        let lhs = self.visit_expr(&expr.lhs)?;
        let rhs = self.visit_expr(&expr.rhs)?;
        match expr.op {
            expr::BinaryOp::Add => lhs.check_add(rhs),
            expr::BinaryOp::Sub => lhs.check_sub(rhs),
            expr::BinaryOp::Mul => lhs.check_mul(rhs),
            expr::BinaryOp::Div => lhs.check_div(rhs),
        }
    }
}

/// Evaluation of the leaf amount, which is the only part [`EvalExpr`] varies on.
trait EvalAmount<'ctx> {
    fn eval_amount(&mut self, amount: &expr::Amount) -> Result<Evaluated<'ctx>, EvalError<'ctx>>;
}

/// [`EvalAmount`] registering the unknown commodities into the context.
struct RegisterCommodity<'a, 'ctx>(&'a mut ReportContext<'ctx>);

impl<'ctx> EvalAmount<'ctx> for RegisterCommodity<'_, 'ctx> {
    fn eval_amount(&mut self, amount: &expr::Amount) -> Result<Evaluated<'ctx>, EvalError<'ctx>> {
        Ok(Evaluated::from_expr_amount_mut(self.0, amount))
    }
}

/// [`EvalAmount`] raising an error on the unknown commodities.
struct ResolveCommodity<'a, 'ctx>(&'a ReportContext<'ctx>);

impl<'ctx> EvalAmount<'ctx> for ResolveCommodity<'_, 'ctx> {
    fn eval_amount(&mut self, amount: &expr::Amount) -> Result<Evaluated<'ctx>, EvalError<'ctx>> {
        Evaluated::from_expr_amount(self.0, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use maplit::btreemap;
    use pretty_assertions::assert_eq;
    use pretty_decimal::PrettyDecimal;
    use rust_decimal_macros::dec;

    #[test]
    fn eval_expr_simple() {
        let input = expr::ValueExpr::Amount(expr::Amount {
            value: PrettyDecimal::plain(dec!(100.12345)),
            commodity: "USD".into(),
        });
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let got = input.eval_mut(&mut ctx).unwrap();
        let got: Amount<'_> = got.try_into().expect("not an amount");
        assert_eq!(
            btreemap! {
                ctx.commodities.ensure("USD") => dec!(100.12345),
            },
            got.into_values()
        );
    }

    #[test]
    fn eval_expr_add_negate() {
        let input = "(100 USD + 300 EUR + (-100 USD + 20,000 JPY))";
        let input: expr::ValueExpr<'static> = input.try_into().expect("must succeed to parse");
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let got = input.eval_mut(&mut ctx).unwrap();
        let got: Amount<'_> = got.try_into().expect("not an amount");
        assert_eq!(
            btreemap! {
                ctx.commodities.ensure("USD") => dec!(0),
                ctx.commodities.ensure("EUR") => dec!(300),
                ctx.commodities.ensure("JPY") => dec!(20000),
            },
            got.into_values()
        );
    }

    #[test]
    fn eval_expr_complex() {
        let input = "((100 USD + 200 EUR) * 2 - 100 USD / 5)";
        let input: expr::ValueExpr = input.try_into().expect("must not fail to parse");
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let got = input.eval_mut(&mut ctx).unwrap();
        let got: Amount<'_> = got.try_into().expect("not an amount");
        assert_eq!(
            btreemap! {
                ctx.commodities.ensure("USD") => dec!(180),
                ctx.commodities.ensure("EUR") => dec!(400),
            },
            got.into_values()
        );
    }
}
