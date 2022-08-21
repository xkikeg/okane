//! Defines value expression representation used in Ledger format.
//! Note this is purely lexicographical and not always valid expression.

use crate::data;

/// Defines value expression.
/// Value expression is a valid expression when used in amount.
/// It can be either amount literal or expression wrapped in `()`.
#[derive(Debug, PartialEq, Eq)]
pub enum ValueExpr {
    Paren(Expr),
    Amount(data::Amount),
}

impl From<data::Amount> for ValueExpr {
    fn from(v: data::Amount) -> Self {
        ValueExpr::Amount(v)
    }
}

/// Generic expression.
#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Unary(UnaryOpExpr),
    Binary(BinaryOpExpr),
    Paren(Box<Expr>),
    Amount(data::Amount),
}

/// Represents unary operator.
#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-x`
    Negate,
}

/// Unary operator expression.
#[derive(Debug, PartialEq, Eq)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

/// Binary operator.
#[derive(Debug, PartialEq, Eq)]
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

/// Represents binary operator expression.
#[derive(Debug, PartialEq, Eq)]
pub struct BinaryOpExpr {
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}
