use std::{
    fmt::Display,
    ops::{Mul, Neg},
};

use rust_decimal::Decimal;

use crate::report::{commodity::CommodityTag, ReportContext};

use super::error::EvalError;

/// Amount with only one commodity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SingleAmount<'ctx> {
    pub(crate) commodity: CommodityTag<'ctx>,
    pub(crate) value: Decimal,
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
    pub fn from_value(commodity: CommodityTag<'ctx>, value: Decimal) -> Self {
        Self { value, commodity }
    }

    /// Adds the amount with keeping commodity single.
    pub fn check_add(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
        if self.commodity != rhs.commodity {
            Err(EvalError::UnmatchingCommodities(
                self.commodity,
                rhs.commodity,
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
    pub fn check_sub(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
        self.check_add(-rhs)
    }

    /// Divides by given Decimal.
    pub fn check_div(self, rhs: Decimal) -> Result<Self, EvalError<'ctx>> {
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

    /// Rounds the Amount with the given context provided precision.
    pub fn round(self, ctx: &ReportContext) -> Self {
        match ctx.commodities.get_decimal_point(self.commodity) {
            None => self,
            Some(dp) => Self {
                value: self.value.round_dp_with_strategy(
                    dp,
                    rust_decimal::RoundingStrategy::MidpointNearestEven,
                ),
                commodity: self.commodity,
            },
        }
    }

    /// Returns a new instance with having the same sign with given SingleAmount.
    pub(crate) fn with_sign_of(mut self, sign: Self) -> Self {
        self.value.set_sign_positive(sign.value.is_sign_positive());
        self
    }

    /// Returns an instance which can be displayed.
    pub fn as_display<'a>(&'a self, ctx: &'a ReportContext<'ctx>) -> impl Display + 'a
    where
        'a: 'ctx,
    {
        SingleAmountDisplay(self, ctx)
    }
}

struct SingleAmountDisplay<'a, 'ctx>(&'a SingleAmount<'ctx>, &'a ReportContext<'ctx>);

impl Display for SingleAmountDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.0.value,
            self.0.commodity.to_str_lossy(&self.1.commodities)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use pretty_assertions::assert_eq;
    use pretty_decimal::PrettyDecimal;
    use rust_decimal_macros::dec;

    use crate::report::ReportContext;

    #[test]
    fn neg_returns_negative_value() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();

        assert_eq!(
            SingleAmount::from_value(jpy, dec!(-5)),
            -SingleAmount::from_value(jpy, dec!(5))
        );
    }

    #[test]
    fn check_add_fails_different_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();
        let chf = ctx.commodities.insert("CHF").unwrap();

        assert_eq!(
            Err(EvalError::UnmatchingCommodities(jpy, chf)),
            SingleAmount::from_value(jpy, dec!(10))
                .check_add(SingleAmount::from_value(chf, dec!(20)))
        );
    }

    #[test]
    fn check_add_succeeds() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();

        assert_eq!(
            SingleAmount::from_value(jpy, dec!(-10)),
            SingleAmount::from_value(jpy, dec!(10))
                .check_add(SingleAmount::from_value(jpy, dec!(-20)))
                .unwrap()
        );
    }

    #[test]
    fn check_sub_fails_different_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();
        let chf = ctx.commodities.insert("CHF").unwrap();

        assert_eq!(
            Err(EvalError::UnmatchingCommodities(jpy, chf)),
            SingleAmount::from_value(jpy, dec!(10))
                .check_sub(SingleAmount::from_value(chf, dec!(0)))
        );
    }

    #[test]
    fn check_sub_succeeds() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();

        assert_eq!(
            SingleAmount::from_value(jpy, dec!(5)),
            SingleAmount::from_value(jpy, dec!(10))
                .check_sub(SingleAmount::from_value(jpy, dec!(5)))
                .unwrap()
        );
    }

    #[test]
    fn single_amount_to_string() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let usd = ctx.commodities.insert("USD").unwrap();

        assert_eq!(
            "1.20 USD".to_string(),
            SingleAmount::from_value(usd, dec!(1.20))
                .as_display(&ctx)
                .to_string()
        );
    }

    #[test]
    fn single_amount_round() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let eur = ctx.commodities.ensure("EUR");
        let chf = ctx.commodities.ensure("CHF");

        ctx.commodities
            .set_format(jpy, PrettyDecimal::comma3dot(dec!(12345)));
        ctx.commodities
            .set_format(eur, PrettyDecimal::plain(dec!(123.45)));
        ctx.commodities
            .set_format(chf, PrettyDecimal::comma3dot(dec!(123.450)));

        // as-is
        assert_eq!(
            SingleAmount::from_value(jpy, dec!(812)),
            SingleAmount::from_value(jpy, dec!(812)).round(&ctx),
        );
        assert_eq!(
            SingleAmount::from_value(eur, dec!(-100.00)),
            SingleAmount::from_value(eur, dec!(-100.0)).round(&ctx),
        );
        assert_eq!(
            SingleAmount::from_value(chf, dec!(6.660)),
            SingleAmount::from_value(chf, dec!(6.66)).round(&ctx),
        );

        assert_eq!(
            SingleAmount::from_value(jpy, dec!(812)),
            SingleAmount::from_value(jpy, dec!(812.5)).round(&ctx),
        );
        assert_eq!(
            SingleAmount::from_value(eur, dec!(-100.02)),
            SingleAmount::from_value(eur, dec!(-100.015)).round(&ctx),
        );
        assert_eq!(
            SingleAmount::from_value(chf, dec!(6.666)),
            SingleAmount::from_value(chf, dec!(6.6665)).round(&ctx),
        );
    }

    #[test]
    fn with_sign_negative() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();
        let eur = ctx.commodities.insert("EUR").unwrap();

        let positive = SingleAmount::from_value(jpy, dec!(1000));
        assert_eq!(
            SingleAmount::from_value(eur, dec!(15)),
            SingleAmount::from_value(eur, dec!(15)).with_sign_of(positive)
        );
        assert_eq!(
            SingleAmount::from_value(eur, dec!(0)),
            SingleAmount::from_value(eur, dec!(0)).with_sign_of(positive)
        );
        assert_eq!(
            SingleAmount::from_value(eur, dec!(15)),
            SingleAmount::from_value(eur, dec!(-15)).with_sign_of(positive)
        );

        let negative = SingleAmount::from_value(jpy, dec!(-1000));
        assert_eq!(
            SingleAmount::from_value(eur, dec!(-15)),
            SingleAmount::from_value(eur, dec!(15)).with_sign_of(negative)
        );
        assert_eq!(
            SingleAmount::from_value(eur, dec!(0)),
            SingleAmount::from_value(eur, dec!(0)).with_sign_of(negative)
        );
        assert_eq!(
            SingleAmount::from_value(eur, dec!(-15)),
            SingleAmount::from_value(eur, dec!(-15)).with_sign_of(negative)
        );
    }
}
