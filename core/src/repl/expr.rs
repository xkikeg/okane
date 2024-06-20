//! Defines value expression representation used in Ledger format.
//! Note this is purely lexicographical and not always valid expression.

use super::pretty_decimal::PrettyDecimal;
use crate::datamodel;

use core::fmt;
use std::borrow::Cow;

/// Amount with presentation information.
/// Similar to `datamodel::Amount` with extra formatting information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount<'i> {
    pub value: PrettyDecimal,
    pub commodity: Cow<'i, str>,
}

impl<'i> From<&'i datamodel::Amount> for Amount<'i> {
    fn from(value: &'i datamodel::Amount) -> Self {
        Self {
            value: PrettyDecimal::unformatted(value.value),
            commodity: Cow::Borrowed(&value.commodity),
        }
    }
}

/// Defines value expression.
/// Value expression is a valid expression when used in amount.
/// It can be either amount literal or expression wrapped in `()`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueExpr<'i> {
    Paren(Expr<'i>),
    Amount(Amount<'i>),
}

impl<'i> From<Amount<'i>> for ValueExpr<'i> {
    fn from(v: Amount<'i>) -> Self {
        ValueExpr::Amount(v)
    }
}

impl<'i> From<&'i datamodel::Amount> for ValueExpr<'i> {
    fn from(value: &'i datamodel::Amount) -> Self {
        ValueExpr::Amount(value.into())
    }
}

/// Generic expression.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr<'i> {
    Unary(UnaryOpExpr<'i>),
    Binary(BinaryOpExpr<'i>),
    Value(Box<ValueExpr<'i>>),
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
pub struct UnaryOpExpr<'i> {
    pub op: UnaryOp,
    pub expr: Box<Expr<'i>>,
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
pub struct BinaryOpExpr<'i> {
    pub op: BinaryOp,
    pub lhs: Box<Expr<'i>>,
    pub rhs: Box<Expr<'i>>,
}
