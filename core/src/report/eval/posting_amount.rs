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
            PostingAmount::Single(single) => write!(f, "{}", single),
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

// impl Mul<Decimal> for PostingAmount<'_> {
//     type Output = Self;

//     fn mul(self, rhs: Decimal) -> Self::Output {
//         match self {
//             PostingAmount::Zero => PostingAmount::Zero,
//             PostingAmount::Single(single) => PostingAmount::Single(single * rhs),
//         }
//     }
// }

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
