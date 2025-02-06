use std::{fmt::Display, ops::Neg};

use super::{error::EvalError, single_amount::SingleAmount};

/// Amount with only one commodity, or total zero.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PostingAmount<'ctx> {
    Zero,
    Single(SingleAmount<'ctx>),
}

impl Default for PostingAmount<'_> {
    fn default() -> Self {
        Self::Zero
    }
}

impl<'ctx> TryFrom<PostingAmount<'ctx>> for SingleAmount<'ctx> {
    type Error = EvalError;

    fn try_from(amount: PostingAmount<'ctx>) -> Result<Self, Self::Error> {
        match amount {
            PostingAmount::Single(single) => Ok(single),
            PostingAmount::Zero => Err(EvalError::SingleAmountRequired),
        }
    }
}

impl<'ctx> From<SingleAmount<'ctx>> for PostingAmount<'ctx> {
    fn from(amount: SingleAmount<'ctx>) -> Self {
        PostingAmount::Single(amount)
    }
}

impl Display for PostingAmount<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostingAmount::Zero => write!(f, "0"),
            PostingAmount::Single(single) => single.fmt(f),
        }
    }
}

impl Neg for PostingAmount<'_> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            PostingAmount::Zero => PostingAmount::Zero,
            PostingAmount::Single(amount) => PostingAmount::Single(-amount),
        }
    }
}

impl PostingAmount<'_> {
    /// Returns absolute zero.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Adds the amount with keeping commodity single.
    pub fn check_add(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (PostingAmount::Zero, _) => Ok(rhs),
            (_, PostingAmount::Zero) => Ok(self),
            (PostingAmount::Single(lhs), PostingAmount::Single(rhs)) => {
                lhs.check_add(rhs).map(Self::Single)
            }
        }
    }

    /// Subtracts the amount with keeping the commodity single.
    pub fn check_sub(self, rhs: Self) -> Result<Self, EvalError> {
        self.check_add(-rhs)
    }
}

#[cfg(test)]
impl<'ctx> PostingAmount<'ctx> {
    /// Constructs an instance with single commodity.
    pub(crate) fn from_value(
        value: rust_decimal::Decimal,
        commodity: crate::report::commodity::Commodity<'ctx>,
    ) -> Self {
        PostingAmount::Single(SingleAmount::from_value(value, commodity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::report::ReportContext;

    #[test]
    fn neg_test() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert_canonical("JPY").unwrap();

        assert_eq!(PostingAmount::Zero, -PostingAmount::zero());

        assert_eq!(
            PostingAmount::from_value(dec!(5), jpy),
            -PostingAmount::from_value(dec!(-5), jpy),
        );
    }

    #[test]
    fn check_add() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert_canonical("JPY").unwrap();

        assert_eq!(
            PostingAmount::from_value(dec!(5), jpy),
            PostingAmount::from_value(dec!(5), jpy)
                .check_add(PostingAmount::zero())
                .unwrap(),
        );

        assert_eq!(
            PostingAmount::from_value(dec!(5), jpy),
            PostingAmount::zero()
                .check_add(PostingAmount::from_value(dec!(5), jpy))
                .unwrap(),
        );
    }
}
