use std::fmt::Display;

use rust_decimal::Decimal;

use crate::{repl::expr, report::ReportContext};

use super::{amount::Amount, error::EvalError};

/// Represents any evaluated value.
#[derive(Debug, PartialEq, Eq)]
pub enum Evaluated<'ctx> {
    Number(Decimal),
    Commodities(Amount<'ctx>),
}

impl<'ctx> TryFrom<Evaluated<'ctx>> for Amount<'ctx> {
    type Error = EvalError;

    fn try_from(value: Evaluated<'ctx>) -> Result<Self, Self::Error> {
        match value {
            Evaluated::Commodities(x) => Ok(x),
            Evaluated::Number(x) if x.is_zero() => Ok(Self::default()),
            _ => Err(EvalError::CommodityAmountRequired),
        }
    }
}

impl<'ctx> From<Decimal> for Evaluated<'ctx> {
    fn from(value: Decimal) -> Self {
        Evaluated::Number(value)
    }
}

impl<'ctx> From<Amount<'ctx>> for Evaluated<'ctx> {
    fn from(value: Amount<'ctx>) -> Self {
        Evaluated::Commodities(value)
    }
}

impl<'ctx> Evaluated<'ctx> {
    /// Creates Evaluated from [expr::Amount].
    pub(super) fn from_expr_amount(
        ctx: &mut ReportContext<'ctx>,
        amount: &expr::Amount,
    ) -> Evaluated<'ctx> {
        if amount.commodity.is_empty() {
            return amount.value.value.into();
        }
        let commodity = ctx.commodities.intern(&amount.commodity);
        Amount::from_value(amount.value.clone().into(), commodity).into()
    }

    /// Returns if the amount is zero.
    pub fn is_zero(&self) -> bool {
        match self {
            Evaluated::Number(x) => x.is_zero(),
            Evaluated::Commodities(y) => y.is_zero(),
        }
    }

    /// Returns negative signed self.
    pub fn negate(self) -> Self {
        match self {
            Evaluated::Number(x) => Evaluated::Number(-x),
            Evaluated::Commodities(x) => Evaluated::Commodities(x.negate()),
        }
    }

    /// Returns `self + rhs`, or error.
    /// Operation with the following types supported.
    /// * number + number
    /// * commodities + commodities
    pub fn check_add(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Evaluated::Number(l), Evaluated::Number(r)) => Ok(Evaluated::Number(l + r)),
            (Evaluated::Commodities(l), Evaluated::Commodities(r)) => {
                Ok(Evaluated::Commodities(l + r))
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }

    /// Returns `self - rhs`, or error.
    /// Operation with the following types supported.
    /// * number - number
    /// * commodities - commodities
    pub fn check_sub(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Evaluated::Number(l), Evaluated::Number(r)) => Ok(Evaluated::Number(l - r)),
            (Evaluated::Commodities(l), Evaluated::Commodities(r)) => {
                Ok(Evaluated::Commodities(l - r))
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }

    /// Returns `self * rhs`, or error.
    /// Operation with the following types supported.
    /// * number * number
    /// * commodities * number
    /// * number * commodities
    pub fn check_mul(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Evaluated::Number(x), Evaluated::Number(y)) => Ok(Evaluated::Number(x * y)),
            (Evaluated::Commodities(x), Evaluated::Number(y)) => Ok(Evaluated::Commodities(x * y)),
            (Evaluated::Number(x), Evaluated::Commodities(y)) => Ok(Evaluated::Commodities(y * x)),
            _ => Err(EvalError::UnmatchingOperation),
        }
    }

    /// Returns `self / rhs`, or error.
    /// Operation with the following types supported.
    /// * number / number
    /// * commodities / number
    pub fn check_div(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (_, rhs) if rhs.is_zero() => Err(EvalError::DivideByZero),
            (Evaluated::Number(x), Evaluated::Number(y)) => Ok(Evaluated::Number(x / y)),
            (Evaluated::Commodities(x), Evaluated::Number(y)) => {
                x.check_div(y).map(Evaluated::Commodities)
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }
}

impl<'ctx> Display for Evaluated<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Number(x) => x.fmt(f),
            Evaluated::Commodities(x) => x.as_inline_display().fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_into_amount() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert_eq!(
            Amount::try_from(Evaluated::from(dec!(5))).unwrap_err(),
            EvalError::CommodityAmountRequired
        );

        assert_eq!(
            Amount::try_from(Evaluated::from(dec!(0))).unwrap(),
            Amount::zero(),
        );

        assert_eq!(
            Amount::try_from(Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("USD")
            )))
            .unwrap(),
            Amount::from_value(dec!(1000), ctx.commodities.intern("USD"))
        );
    }

    #[test]
    fn test_is_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert!(Evaluated::from(dec!(0)).is_zero());
        assert!(!Evaluated::from(dec!(1.5)).is_zero());

        assert!(Evaluated::from(Amount::zero()).is_zero());
        assert!(
            Evaluated::from(Amount::from_value(dec!(0), ctx.commodities.intern("USD"))).is_zero()
        );
        assert!(!Evaluated::from(Amount::from_value(
            dec!(1000),
            ctx.commodities.intern("USD")
        ))
        .is_zero());
    }

    #[test]
    fn test_negate() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert_eq!(
            Evaluated::from(dec!(1.5)).negate(),
            Evaluated::from(dec!(-1.5))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("USD")
            ))
            .negate(),
            Evaluated::from(Amount::from_value(
                dec!(-1000),
                ctx.commodities.intern("USD")
            ))
        );
    }

    #[test]
    fn test_check_add() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_add(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(3.0))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
            .check_add(Evaluated::from(Amount::from_value(
                dec!(100),
                ctx.commodities.intern("USD")
            )))
            .unwrap(),
            Evaluated::from(Amount::from_values([
                (dec!(1000), ctx.commodities.intern("JPY")),
                (dec!(100), ctx.commodities.intern("USD")),
            ]))
        );

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_add(Evaluated::from(Amount::from_value(
                    dec!(1000),
                    ctx.commodities.intern("JPY")
                )))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
            .check_add(Evaluated::from(dec!(1)))
            .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }

    #[test]
    fn test_check_sub() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_sub(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(-1.0))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
            .check_sub(Evaluated::from(Amount::from_value(
                dec!(100),
                ctx.commodities.intern("USD")
            )))
            .unwrap(),
            Evaluated::from(Amount::from_values([
                (dec!(1000), ctx.commodities.intern("JPY")),
                (dec!(-100), ctx.commodities.intern("USD")),
            ]))
        );

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_sub(Evaluated::from(Amount::from_value(
                    dec!(1000),
                    ctx.commodities.intern("JPY")
                )))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
            .check_sub(Evaluated::from(dec!(1)))
            .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }

    #[test]
    fn test_check_mul() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert_eq!(
            Evaluated::from(dec!(1.5))
                .check_mul(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(3.0))
        );

        assert_eq!(
            Evaluated::from(dec!(-0.2))
                .check_mul(Evaluated::from(Amount::from_value(
                    dec!(1000),
                    ctx.commodities.intern("JPY")
                )))
                .unwrap(),
            Evaluated::from(Amount::from_value(
                dec!(-200),
                ctx.commodities.intern("JPY")
            ))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(3.15),
                ctx.commodities.intern("USD")
            ))
            .check_mul(Evaluated::from(dec!(0.5)))
            .unwrap(),
            Evaluated::from(Amount::from_value(
                dec!(1.575),
                ctx.commodities.intern("USD")
            ))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
            .check_mul(Evaluated::from(Amount::from_value(
                dec!(100),
                ctx.commodities.intern("USD")
            )))
            .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }

    #[test]
    fn test_check_div() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        assert_eq!(
            Evaluated::from(dec!(1.5))
                .check_div(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(0.75))
        );

        assert_eq!(
            Evaluated::from(dec!(1.5))
                .check_div(Evaluated::from(dec!(0)))
                .unwrap_err(),
            EvalError::DivideByZero
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(3.15),
                ctx.commodities.intern("USD")
            ))
            .check_div(Evaluated::from(dec!(0.5)))
            .unwrap(),
            Evaluated::from(Amount::from_value(
                dec!(6.30),
                ctx.commodities.intern("USD")
            ))
        );

        assert_eq!(
            Evaluated::from(dec!(-0.2))
                .check_div(Evaluated::from(Amount::from_value(
                    dec!(1000),
                    ctx.commodities.intern("JPY")
                )))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );

        // Division across the same currency could be supported,
        // but it's cumbersome with little value.
        assert_eq!(
            Evaluated::from(Amount::from_value(
                dec!(1000),
                ctx.commodities.intern("JPY")
            ))
            .check_div(Evaluated::from(Amount::from_value(
                dec!(100),
                ctx.commodities.intern("JPY")
            )))
            .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }
}
