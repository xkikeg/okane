use std::{
    fmt::Display,
    ops::{Mul, Neg},
};

use rust_decimal::Decimal;

use crate::report::commodity::Commodity;

use super::error::EvalError;

/// Amount with only one commodity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SingleAmount<'ctx> {
    pub(crate) value: Decimal,
    pub(crate) commodity: Commodity<'ctx>,
}

impl Neg for SingleAmount<'_> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        SingleAmount {
            value: -self.value,
            commodity: self.commodity,
        }
    }
}

impl Mul<Decimal> for SingleAmount<'_> {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self {
            value: self.value * rhs,
            commodity: self.commodity,
        }
    }
}

impl<'ctx> SingleAmount<'ctx> {
    /// Constructs an instance with single commodity.
    #[inline]
    pub fn from_value(value: Decimal, commodity: Commodity<'ctx>) -> Self {
        Self { value, commodity }
    }

    /// Adds the amount with keeping commodity single.
    pub fn check_add(self, rhs: Self) -> Result<Self, EvalError> {
        if self.commodity != rhs.commodity {
            Err(EvalError::UnmatchingCommodities(
                self.commodity.into(),
                rhs.commodity.into(),
            ))
        } else {
            Ok(Self {
                value: self
                    .value
                    .checked_add(rhs.value)
                    .ok_or(EvalError::NumberOverflow)?,
                commodity: self.commodity,
            })
        }
    }

    /// Subtracts the amount with keeping the commodity single.
    pub fn check_sub(self, rhs: Self) -> Result<Self, EvalError> {
        self.check_add(-rhs)
    }

    /// Divides by given Decimal.
    pub fn check_div(self, rhs: Decimal) -> Result<Self, EvalError> {
        if rhs.is_zero() {
            return Err(EvalError::DivideByZero);
        }
        Ok(Self {
            value: self
                .value
                .checked_div(rhs)
                .ok_or(EvalError::NumberOverflow)?,
            commodity: self.commodity,
        })
    }

    /// Returns an absolute value of the current value.
    pub fn abs(self) -> Self {
        Self {
            value: self.value.abs(),
            commodity: self.commodity,
        }
    }

    /// Returns a new instance with having the same sign with given SingleAmount.
    pub(crate) fn with_sign_of(self, rhs: Self) -> Self {
        let value = if rhs.value.is_sign_positive() {
            self.value
        } else {
            self.value.neg()
        };
        Self {
            value,
            commodity: self.commodity,
        }
    }
}

impl Display for SingleAmount<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.commodity.as_str())
    }
}
