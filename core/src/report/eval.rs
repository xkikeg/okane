//! Evaluation logics of expr.

mod amount;
mod error;
mod evaluated;
mod posting_amount;
mod single_amount;

pub use amount::Amount;
pub use error::EvalError;
pub use evaluated::Evaluated;
pub(super) use posting_amount::PostingAmount;
pub use single_amount::SingleAmount;

use super::context::ReportContext;
use crate::syntax::expr;

// Provides evaluation to syntax expressions.
pub(crate) trait Evaluable {
    fn eval_visit<'ctx, F: FnMut(&expr::Amount) -> Result<Evaluated<'ctx>, EvalError>>(
        &self,
        evaluator: &mut F,
    ) -> Result<Evaluated<'ctx>, EvalError>;

    /// Evaluate the self with mutable `ctx`, which allows unknown commodities in the expressions to be registered.
    fn eval_mut<'ctx>(&self, ctx: &mut ReportContext<'ctx>) -> Result<Evaluated<'ctx>, EvalError> {
        self.eval_visit(&mut |amount| Ok(Evaluated::from_expr_amount_mut(ctx, amount)))
    }

    /// Evaluate the self with immutable `ctx`, which raises error on unknown commoditieis.
    fn eval<'ctx>(&self, ctx: &ReportContext<'ctx>) -> Result<Evaluated<'ctx>, EvalError> {
        self.eval_visit(&mut |amount| Evaluated::from_expr_amount(ctx, amount))
    }
}

impl Evaluable for expr::ValueExpr<'_> {
    fn eval_visit<'ctx, F: FnMut(&expr::Amount) -> Result<Evaluated<'ctx>, EvalError>>(
        &self,
        evaluator: &mut F,
    ) -> Result<Evaluated<'ctx>, EvalError> {
        match self {
            expr::ValueExpr::Paren(x) => x.eval_visit(evaluator),
            expr::ValueExpr::Amount(x) => evaluator(x),
        }
    }
}

impl Evaluable for expr::Expr<'_> {
    fn eval_visit<'ctx, F: FnMut(&expr::Amount) -> Result<Evaluated<'ctx>, EvalError>>(
        &self,
        evaluator: &mut F,
    ) -> Result<Evaluated<'ctx>, EvalError> {
        match self {
            expr::Expr::Unary(e) => e.eval_visit(evaluator),
            expr::Expr::Binary(e) => e.eval_visit(evaluator),
            expr::Expr::Value(e) => e.eval_visit(evaluator),
        }
    }
}

impl Evaluable for expr::UnaryOpExpr<'_> {
    fn eval_visit<'ctx, F: FnMut(&expr::Amount) -> Result<Evaluated<'ctx>, EvalError>>(
        &self,
        evaluator: &mut F,
    ) -> Result<Evaluated<'ctx>, EvalError> {
        match self.op {
            expr::UnaryOp::Negate => {
                let val = self.expr.eval_visit(evaluator)?;
                Ok(val.negate())
            }
        }
    }
}

impl Evaluable for expr::BinaryOpExpr<'_> {
    fn eval_visit<'ctx, F: FnMut(&expr::Amount) -> Result<Evaluated<'ctx>, EvalError>>(
        &self,
        evaluator: &mut F,
    ) -> Result<Evaluated<'ctx>, EvalError> {
        let lhs = self.lhs.eval_visit(evaluator)?;
        let rhs = self.rhs.eval_visit(evaluator)?;
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
            hashmap! {
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
            hashmap! {
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
            hashmap! {
                ctx.commodities.ensure("USD") => dec!(180),
                ctx.commodities.ensure("EUR") => dec!(400),
            },
            got.into_values()
        );
    }
}
