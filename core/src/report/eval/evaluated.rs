use std::fmt::Display;

use rust_decimal::Decimal;

use crate::{
    report::{commodity::OwnedCommodity, ReportContext},
    syntax::expr,
};

use super::{error::EvalError, Amount, PostingAmount, SingleAmount};

/// Represents any evaluated value.
#[derive(Debug, PartialEq, Eq)]
pub enum Evaluated<'ctx> {
    Number(Decimal),
    Commodities(Amount<'ctx>),
}

impl<'ctx> TryFrom<Evaluated<'ctx>> for SingleAmount<'ctx> {
    type Error = EvalError<'ctx>;

    fn try_from(value: Evaluated<'ctx>) -> Result<Self, Self::Error> {
        let amount: Amount<'ctx> = value.try_into()?;
        amount.try_into()
    }
}

impl<'ctx> TryFrom<Evaluated<'ctx>> for PostingAmount<'ctx> {
    type Error = EvalError<'ctx>;

    fn try_from(value: Evaluated<'ctx>) -> Result<Self, Self::Error> {
        let amount: Amount<'ctx> = value.try_into()?;
        amount.try_into()
    }
}

impl<'ctx> TryFrom<Evaluated<'ctx>> for Amount<'ctx> {
    type Error = EvalError<'ctx>;

    fn try_from(value: Evaluated<'ctx>) -> Result<Self, Self::Error> {
        match value {
            Evaluated::Commodities(x) => Ok(x),
            Evaluated::Number(x) if x.is_zero() => Ok(Self::default()),
            _ => Err(EvalError::AmountRequired),
        }
    }
}

impl From<Decimal> for Evaluated<'_> {
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
    /// Creates [`Evaluated`] from [`expr::Amount`],
    /// with registering the commodity.
    pub(super) fn from_expr_amount_mut(
        ctx: &mut ReportContext<'ctx>,
        amount: &expr::Amount,
    ) -> Evaluated<'ctx> {
        if amount.commodity.is_empty() {
            return amount.value.value.into();
        }
        let commodity = ctx.commodities.ensure(&amount.commodity);
        Amount::from_value(commodity, amount.value.into()).into()
    }

    /// Creates [`Evaluated`] from [`expr::Amount`],
    /// with just looking up the commodity.
    pub(super) fn from_expr_amount<'a>(
        ctx: &ReportContext<'ctx>,
        amount: &'a expr::Amount<'a>,
    ) -> Result<Evaluated<'ctx>, EvalError<'ctx>> {
        if amount.commodity.is_empty() {
            return Ok(amount.value.value.into());
        }
        let commodity = ctx.commodities.resolve(&amount.commodity).ok_or_else(|| {
            EvalError::UnknownCommodity(OwnedCommodity::from_string(
                amount.commodity.clone().into_owned(),
            ))
        })?;
        Ok(Amount::from_value(commodity, amount.value.into()).into())
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
    pub fn check_add(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
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
    pub fn check_sub(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
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
    pub fn check_mul(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
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
    pub fn check_div(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
        match (self, rhs) {
            (_, rhs) if rhs.is_zero() => Err(EvalError::DivideByZero),
            (Evaluated::Number(x), Evaluated::Number(y)) => Ok(Evaluated::Number(x / y)),
            (Evaluated::Commodities(x), Evaluated::Number(y)) => {
                x.check_div(y).map(Evaluated::Commodities)
            }
            (Evaluated::Number(x), Evaluated::Commodities(y)) => {
                let y: SingleAmount = y.try_into()?;
                let ret = SingleAmount::from_value(y.commodity, x).check_div(y.value)?;
                Ok(Evaluated::Commodities(ret.into()))
            }
            _ => Err(EvalError::UnmatchingOperation),
        }
    }

    /// Returns display impl.
    pub fn as_display<'a>(&'a self, ctx: &'a ReportContext<'ctx>) -> impl Display + 'a
    where
        'a: 'ctx,
    {
        EvaluatedDisplay(self, ctx)
    }
}

struct EvaluatedDisplay<'a, 'ctx>(&'a Evaluated<'ctx>, &'a ReportContext<'ctx>);

impl Display for EvaluatedDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Evaluated::Number(x) => x.fmt(f),
            Evaluated::Commodities(x) => x.as_inline_display(self.1).fmt(f),
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
        let usd = ctx.commodities.ensure("USD");

        assert_eq!(
            Amount::try_from(Evaluated::from(dec!(5))).unwrap_err(),
            EvalError::AmountRequired
        );

        assert_eq!(
            Amount::try_from(Evaluated::from(dec!(0))).unwrap(),
            Amount::zero(),
        );

        assert_eq!(
            Amount::try_from(Evaluated::from(Amount::from_value(usd, dec!(1000)))).unwrap(),
            Amount::from_value(usd, dec!(1000))
        );
    }

    #[test]
    fn test_display() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodities.ensure("USD");

        assert_eq!("0", &Evaluated::from(dec!(0)).as_display(&ctx).to_string());
        assert_eq!(
            "1.5",
            &Evaluated::from(dec!(1.5)).as_display(&ctx).to_string()
        );

        assert_eq!(
            "0 USD",
            Evaluated::from(Amount::from_value(usd, dec!(0)))
                .as_display(&ctx)
                .to_string()
        );
    }

    #[test]
    fn test_is_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodities.ensure("USD");

        assert!(Evaluated::from(dec!(0)).is_zero());
        assert!(!Evaluated::from(dec!(1.5)).is_zero());

        assert!(Evaluated::from(Amount::zero()).is_zero());
        assert!(Evaluated::from(Amount::from_value(usd, dec!(0))).is_zero());
        assert!(!Evaluated::from(Amount::from_value(usd, dec!(1000))).is_zero());
    }

    #[test]
    fn test_negate() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodities.ensure("USD");

        assert_eq!(
            Evaluated::from(dec!(1.5)).negate(),
            Evaluated::from(dec!(-1.5))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(usd, dec!(1000))).negate(),
            Evaluated::from(Amount::from_value(usd, dec!(-1000)))
        );
    }

    #[test]
    fn test_check_add() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_add(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(3.0))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(jpy, dec!(1000)))
                .check_add(Evaluated::from(Amount::from_value(usd, dec!(100))))
                .unwrap(),
            Evaluated::from(Amount::from_iter([(jpy, dec!(1000)), (usd, dec!(100)),]))
        );

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_add(Evaluated::from(Amount::from_value(jpy, dec!(1000))))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
        assert_eq!(
            Evaluated::from(Amount::from_value(jpy, dec!(1000)))
                .check_add(Evaluated::from(dec!(1)))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }

    #[test]
    fn test_check_sub() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_sub(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(-1.0))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(jpy, dec!(1000)))
                .check_sub(Evaluated::from(Amount::from_value(usd, dec!(100))))
                .unwrap(),
            Evaluated::from(Amount::from_iter([(jpy, dec!(1000)), (usd, dec!(-100)),]))
        );

        assert_eq!(
            Evaluated::from(dec!(1))
                .check_sub(Evaluated::from(Amount::from_value(jpy, dec!(1000))))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
        assert_eq!(
            Evaluated::from(Amount::from_value(jpy, dec!(1000)))
                .check_sub(Evaluated::from(dec!(1)))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }

    #[test]
    fn test_check_mul() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert_eq!(
            Evaluated::from(dec!(1.5))
                .check_mul(Evaluated::from(dec!(2.0)))
                .unwrap(),
            Evaluated::from(dec!(3.0))
        );

        assert_eq!(
            Evaluated::from(dec!(-0.2))
                .check_mul(Evaluated::from(Amount::from_value(jpy, dec!(1000))))
                .unwrap(),
            Evaluated::from(Amount::from_value(jpy, dec!(-200)))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(usd, dec!(3.15)))
                .check_mul(Evaluated::from(dec!(0.5)))
                .unwrap(),
            Evaluated::from(Amount::from_value(usd, dec!(1.575)))
        );

        assert_eq!(
            Evaluated::from(Amount::from_value(jpy, dec!(1000)))
                .check_mul(Evaluated::from(Amount::from_value(usd, dec!(100))))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }

    #[test]
    fn test_check_div() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

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
            Evaluated::from(Amount::from_value(usd, dec!(3.15)))
                .check_div(Evaluated::from(dec!(0.5)))
                .unwrap(),
            Evaluated::from(Amount::from_value(usd, dec!(6.30)))
        );

        assert_eq!(
            Evaluated::from(dec!(-0.2))
                .check_div(Evaluated::from(Amount::from_value(jpy, dec!(100))))
                .unwrap(),
            Evaluated::from(Amount::from_value(jpy, dec!(-0.002)))
        );

        // Division across the same currency could be supported,
        // but it's cumbersome with little value.
        assert_eq!(
            Evaluated::from(Amount::from_value(jpy, dec!(1000)))
                .check_div(Evaluated::from(Amount::from_value(jpy, dec!(100))))
                .unwrap_err(),
            EvalError::UnmatchingOperation
        );
    }
}
