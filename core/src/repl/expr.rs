//! Defines value expression representation used in Ledger format.
//! Note this is purely lexicographical and not always valid expression.

use super::pretty_decimal::PrettyDecimal;
use crate::datamodel;

use core::fmt;

/// Amount with presentation information.
/// Similar to `datamodel::Amount` with extra formatting information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount {
    pub value: PrettyDecimal,
    pub commodity: String,
}

impl From<datamodel::Amount> for Amount {
    fn from(value: datamodel::Amount) -> Self {
        Self {
            value: PrettyDecimal::unformatted(value.value),
            commodity: value.commodity,
        }
    }
}

/// Defines value expression.
/// Value expression is a valid expression when used in amount.
/// It can be either amount literal or expression wrapped in `()`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueExpr {
    Paren(Expr),
    Amount(Amount),
}

impl From<Amount> for ValueExpr {
    fn from(v: Amount) -> Self {
        ValueExpr::Amount(v)
    }
}

impl From<datamodel::Amount> for ValueExpr {
    fn from(value: datamodel::Amount) -> Self {
        ValueExpr::Amount(value.into())
    }
}

/// Generic expression.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Unary(UnaryOpExpr),
    Binary(BinaryOpExpr),
    Value(Box<ValueExpr>),
}

/// Represents unary operator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

/// Binary operator.
#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BinaryOpExpr {
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}
