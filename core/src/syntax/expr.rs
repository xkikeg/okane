//! Defines value expression representation used in Ledger format.
//! Note this is purely lexicographical and not always valid expression.

use core::fmt;
use std::borrow::Cow;

use bounded_static::ToStatic;
use pretty_decimal::PrettyDecimal;

/// Amount, which is a single unit of value with a commodity.
#[derive(Debug, PartialEq, Eq, Clone, ToStatic)]
pub struct Amount<'i> {
    pub value: PrettyDecimal,
    pub commodity: Cow<'i, str>,
}

/// Defines value expression.
/// Value expression is a valid expression when used in amount.
/// It can be either amount literal or expression wrapped in `()`.
#[derive(Debug, PartialEq, Eq, Clone, ToStatic)]
pub enum ValueExpr<'i> {
    Paren(Expr<'i>),
    Amount(Amount<'i>),
}

impl<'i> From<Amount<'i>> for ValueExpr<'i> {
    fn from(v: Amount<'i>) -> Self {
        ValueExpr::Amount(v)
    }
}

/// Generic expression.
#[derive(Debug, PartialEq, Eq, Clone, ToStatic)]
pub enum Expr<'i> {
    Unary(UnaryOpExpr<'i>),
    Binary(BinaryOpExpr<'i>),
    Value(Box<ValueExpr<'i>>),
}

/// Represents unary operator.
#[derive(Debug, PartialEq, Eq, Clone, Copy, ToStatic)]
pub enum UnaryOp {
    /// `-x`
    Negate,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            UnaryOp::Negate => "-",
        };
        write!(f, "{}", op)
    }
}

/// Unary operator expression.
#[derive(Debug, PartialEq, Eq, Clone, ToStatic)]
pub struct UnaryOpExpr<'i> {
    pub op: UnaryOp,
    pub expr: Box<Expr<'i>>,
}

/// Binary operator.
#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::EnumIter, ToStatic)]
pub enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
        };
        write!(f, "{}", op)
    }
}

/// Represents binary operator expression.
#[derive(Debug, PartialEq, Eq, Clone, ToStatic)]
pub struct BinaryOpExpr<'i> {
    pub op: BinaryOp,
    pub lhs: Box<Expr<'i>>,
    pub rhs: Box<Expr<'i>>,
}

/// Visitor over the value expression AST.
///
/// Every `visit_*` method is responsible for recursing into its own children, usually by
/// calling [`ExprVisitor::visit_expr`]. That leaves the visitor free to run code before,
/// between and after the children, which a plain fold over already-computed child values
/// could not express.
///
/// [`ExprVisitor::visit_value_expr`] and [`ExprVisitor::visit_expr`] only dispatch on the node
/// kind, so they come with default implementations. Implementors normally provide just
/// [`ExprVisitor::visit_amount`], [`ExprVisitor::visit_unary`] and [`ExprVisitor::visit_binary`].
pub(crate) trait ExprVisitor<'i> {
    /// Value produced for every expression node.
    type Output;

    /// Error aborting the traversal.
    type Error;

    /// Visits an amount literal, the only leaf of the tree.
    fn visit_amount(&mut self, amount: &Amount<'i>) -> Result<Self::Output, Self::Error>;

    /// Visits a unary operator expression, recursing into `expr.expr`.
    fn visit_unary(&mut self, expr: &UnaryOpExpr<'i>) -> Result<Self::Output, Self::Error>;

    /// Visits a binary operator expression, recursing into `expr.lhs` and `expr.rhs`.
    fn visit_binary(&mut self, expr: &BinaryOpExpr<'i>) -> Result<Self::Output, Self::Error>;

    /// Visits a value expression, dispatching to [`Self::visit_expr`] or [`Self::visit_amount`].
    #[inline]
    fn visit_value_expr(&mut self, expr: &ValueExpr<'i>) -> Result<Self::Output, Self::Error> {
        match expr {
            ValueExpr::Paren(x) => self.visit_expr(x),
            ValueExpr::Amount(x) => self.visit_amount(x),
        }
    }

    /// Visits a generic expression, dispatching on the node kind.
    #[inline]
    fn visit_expr(&mut self, expr: &Expr<'i>) -> Result<Self::Output, Self::Error> {
        match expr {
            Expr::Unary(e) => self.visit_unary(e),
            Expr::Binary(e) => self.visit_binary(e),
            Expr::Value(e) => self.visit_value_expr(e),
        }
    }
}

/// Node of the expression AST an [`ExprVisitor`] can be applied to.
pub(crate) trait Visitable<'i> {
    /// Invokes the `visitor` method corresponding to `Self`.
    fn accept<V: ExprVisitor<'i> + ?Sized>(&self, visitor: &mut V) -> Result<V::Output, V::Error>;
}

impl<'i> Visitable<'i> for Amount<'i> {
    #[inline]
    fn accept<V: ExprVisitor<'i> + ?Sized>(&self, visitor: &mut V) -> Result<V::Output, V::Error> {
        visitor.visit_amount(self)
    }
}

impl<'i> Visitable<'i> for ValueExpr<'i> {
    #[inline]
    fn accept<V: ExprVisitor<'i> + ?Sized>(&self, visitor: &mut V) -> Result<V::Output, V::Error> {
        visitor.visit_value_expr(self)
    }
}

impl<'i> Visitable<'i> for Expr<'i> {
    #[inline]
    fn accept<V: ExprVisitor<'i> + ?Sized>(&self, visitor: &mut V) -> Result<V::Output, V::Error> {
        visitor.visit_expr(self)
    }
}

impl<'i> Visitable<'i> for UnaryOpExpr<'i> {
    #[inline]
    fn accept<V: ExprVisitor<'i> + ?Sized>(&self, visitor: &mut V) -> Result<V::Output, V::Error> {
        visitor.visit_unary(self)
    }
}

impl<'i> Visitable<'i> for BinaryOpExpr<'i> {
    #[inline]
    fn accept<V: ExprVisitor<'i> + ?Sized>(&self, visitor: &mut V) -> Result<V::Output, V::Error> {
        visitor.visit_binary(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use pretty_assertions::assert_eq;

    /// [`ExprVisitor`] recording every node it reaches, relying on the default dispatch.
    #[derive(Default)]
    struct TraceVisitor(Vec<String>);

    impl<'i> ExprVisitor<'i> for TraceVisitor {
        type Output = ();
        type Error = Infallible;

        fn visit_amount(&mut self, amount: &Amount<'i>) -> Result<(), Infallible> {
            self.0.push(format!("amount({})", amount.commodity));
            Ok(())
        }

        fn visit_unary(&mut self, expr: &UnaryOpExpr<'i>) -> Result<(), Infallible> {
            self.0.push(format!("unary({})", expr.op));
            self.visit_expr(&expr.expr)
        }

        fn visit_binary(&mut self, expr: &BinaryOpExpr<'i>) -> Result<(), Infallible> {
            self.0.push(format!("binary({})", expr.op));
            self.visit_expr(&expr.lhs)?;
            self.visit_expr(&expr.rhs)
        }
    }

    #[test]
    fn visitor_reaches_all_nodes_in_pre_order() {
        let input: ValueExpr = "(1 USD * (-2 + 3 EUR))".try_into().expect("must parse");
        let mut visitor = TraceVisitor::default();
        input.accept(&mut visitor).expect("must not fail");
        assert_eq!(
            vec![
                "binary(*)",
                "amount(USD)",
                "binary(+)",
                "unary(-)",
                "amount()",
                "amount(EUR)",
            ],
            visitor.0
        );
    }

    #[test]
    fn visitor_aborts_on_error() {
        /// [`ExprVisitor`] failing on the first amount without a commodity.
        struct FailingVisitor(usize);

        impl<'i> ExprVisitor<'i> for FailingVisitor {
            type Output = ();
            type Error = ();

            fn visit_amount(&mut self, amount: &Amount<'i>) -> Result<(), ()> {
                self.0 += 1;
                if amount.commodity.is_empty() {
                    return Err(());
                }
                Ok(())
            }

            fn visit_unary(&mut self, expr: &UnaryOpExpr<'i>) -> Result<(), ()> {
                self.visit_expr(&expr.expr)
            }

            fn visit_binary(&mut self, expr: &BinaryOpExpr<'i>) -> Result<(), ()> {
                self.visit_expr(&expr.lhs)?;
                self.visit_expr(&expr.rhs)
            }
        }

        let input: ValueExpr = "(1 USD + 2 + 3 EUR)".try_into().expect("must parse");
        let mut visitor = FailingVisitor(0);
        assert_eq!(Err(()), input.accept(&mut visitor));
        // Stopped right at the bare `2`, without reaching `3 EUR`.
        assert_eq!(2, visitor.0);
    }
}
