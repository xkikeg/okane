use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use rust_decimal::Decimal;

use crate::report::context::Commodity;

use super::error::EvalError;

/// Amount with multiple commodities, or simple zero.
// TODO: Rename it to ValueAmount.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Amount<'ctx> {
    // if values.len == zero, then it'll be completely zero.
    // TODO: Consider optimizing for small number of commodities,
    // as most of the case it needs to be just a few elements.
    values: HashMap<Commodity<'ctx>, Decimal>,
}

impl<'ctx> Amount<'ctx> {
    /// Creates an [Amount] with zero value.
    #[inline(always)]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Creates an [Amount] with single value and commodity.
    pub fn from_value(amount: Decimal, commodity: Commodity<'ctx>) -> Self {
        let mut value = Amount::default();
        value.values.insert(commodity, amount);
        value
    }

    /// Creates an [Amount] from a set of values.
    pub fn from_values<T>(values: T) -> Self
    where
        T: IntoIterator<Item = (Decimal, Commodity<'ctx>)>,
    {
        Amount {
            values: values.into_iter().map(|(v, c)| (c, v)).collect(),
        }
    }

    /// Takes out the instance and returns map from commodity to its value.
    pub fn into_values(self) -> HashMap<Commodity<'ctx>, Decimal> {
        self.values
    }

    /// Returns an objectt to print the amount as inline.
    pub fn as_inline_display(&self) -> impl Display + '_ {
        InlinePrintAmount(self)
    }

    /// Returns `true` if this is 'non-commoditized zero', which is used to assert
    /// the account balance is completely zero.
    pub fn is_absolute_zero(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns `true` if this is zero, including zero commodities.
    pub fn is_zero(&self) -> bool {
        self.values.iter().all(|(_, v)| v.is_zero())
    }

    pub fn negate(mut self) -> Self {
        for (_, v) in self.values.iter_mut() {
            v.set_sign_positive(!v.is_sign_positive())
        }
        self
    }

    pub fn check_div(mut self, rhs: Decimal) -> Result<Self, EvalError> {
        if rhs.is_zero() {
            return Err(EvalError::DivideByZero);
        }
        for (_, v) in self.values.iter_mut() {
            *v = v.checked_div(rhs).ok_or(EvalError::NumberOverflow)?;
        }
        Ok(self)
    }
}

#[derive(Debug)]
struct InlinePrintAmount<'a, 'ctx>(&'a Amount<'ctx>);

impl<'a, 'ctx> Display for InlinePrintAmount<'a, 'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vs = &self.0.values;
        match vs.len() {
            0 | 1 => match vs.iter().next() {
                Some((c, v)) => write!(f, "{} {}", v, c.as_str()),
                None => write!(f, "0"),
            },
            _ => {
                write!(f, "(")?;
                for (i, (c, v)) in vs.iter().enumerate() {
                    if i != 0 {
                        write!(f, " + ")?;
                    }
                    write!(f, "{} {}", v, c.as_str())?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<'ctx> Neg for Amount<'ctx> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

impl<'ctx> Add for Amount<'ctx> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'ctx> AddAssign for Amount<'ctx> {
    fn add_assign(&mut self, rhs: Self) {
        for (c, v2) in rhs.values {
            let mut v1 = self.values.entry(c).or_insert(Decimal::ZERO);
            v1 += v2;
            // it's questionable if we should eliminate zero commodities,
            // but it should be safe as non-recorded commotiy and zero commodity
            // don't have behavior difference.
            if v1.is_zero() {
                self.values.remove(&c);
            }
        }
    }
}

impl<'ctx> Sub for Amount<'ctx> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'ctx> SubAssign for Amount<'ctx> {
    fn sub_assign(&mut self, rhs: Self) {
        for (c, v2) in rhs.values {
            let mut v1 = self.values.entry(c).or_insert(Decimal::ZERO);
            v1 -= v2;
            if v1.is_zero() {
                self.values.remove(&c);
            }
        }
    }
}

impl<'ctx> Mul<Decimal> for Amount<'ctx> {
    type Output = Self;

    fn mul(mut self, rhs: Decimal) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'ctx> MulAssign<Decimal> for Amount<'ctx> {
    fn mul_assign(&mut self, rhs: Decimal) {
        if rhs.is_zero() {
            self.values.clear();
            return;
        }
        for (_, mut v) in self.values.iter_mut() {
            v *= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::report::ReportContext;

    use super::*;

    #[test]
    fn test_default() {
        let amount = Amount::default();
        assert_eq!(format!("{}", amount.as_inline_display()), "0")
    }

    #[test]
    fn test_from_value() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let amount = Amount::from_value(dec!(123.45), jpy);
        assert_eq!(format!("{}", amount.as_inline_display()), "123.45 JPY")
    }

    #[test]
    fn test_is_absolute_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert!(Amount::default().is_absolute_zero());
        assert!(!Amount::from_value(dec!(0), jpy).is_absolute_zero());
        assert!(!Amount::from_values([(dec!(0), jpy), (dec!(0), usd)]).is_absolute_zero())
    }

    #[test]
    fn test_is_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert!(Amount::default().is_zero());
        assert!(Amount::from_value(dec!(0), jpy).is_zero());
        assert!(Amount::from_values([(dec!(0), jpy), (dec!(0), usd)]).is_zero());

        assert!(!Amount::from_value(dec!(1), jpy).is_zero());
        assert!(!Amount::from_values([(dec!(0), jpy), (dec!(1), usd)]).is_zero());
    }

    #[test]
    fn test_neg() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert_eq!(-Amount::zero(), Amount::zero());
        assert_eq!(
            -Amount::from_value(dec!(100), jpy),
            Amount::from_value(dec!(-100), jpy)
        );
        assert_eq!(
            -Amount::from_values([(dec!(100), jpy), (dec!(-20.35), usd)]),
            Amount::from_values([(dec!(-100), jpy), (dec!(20.35), usd)]),
        );
    }

    #[test]
    fn test_add() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");
        let eur = ctx.commodities.ensure("EUR");
        let chf = ctx.commodities.ensure("CHF");

        let zero_plus_zero = Amount::zero() + Amount::zero();
        assert_eq!(zero_plus_zero, Amount::zero());

        assert_eq!(
            Amount::from_value(dec!(1), jpy) + Amount::zero(),
            Amount::from_value(dec!(1), jpy),
        );
        assert_eq!(
            Amount::zero() + Amount::from_value(dec!(1), jpy),
            Amount::from_value(dec!(1), jpy),
        );
        assert_eq!(
            Amount::from_values([(dec!(123.00), jpy), (dec!(456.0), usd), (dec!(7.89), eur),]),
            Amount::from_value(dec!(123.45), jpy)
                + Amount::from_value(dec!(-0.45), jpy)
                + Amount::from_value(dec!(456), usd)
                + Amount::from_value(dec!(0.0), usd)
                + -Amount::from_value(dec!(100), chf)
                + Amount::from_value(dec!(7.89), eur)
                + Amount::from_value(dec!(100), chf),
        );
    }

    #[test]
    fn test_sub() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");
        let eur = ctx.commodities.ensure("EUR");
        let chf = ctx.commodities.ensure("CHF");

        let zero_minus_zero = Amount::zero() - Amount::zero();
        assert_eq!(zero_minus_zero, Amount::zero());

        assert_eq!(
            Amount::from_value(dec!(1), jpy) - Amount::zero(),
            Amount::from_value(dec!(1), jpy),
        );
        assert_eq!(
            Amount::zero() - Amount::from_value(dec!(1), jpy),
            Amount::from_value(dec!(-1), jpy),
        );
        assert_eq!(
            Amount::from_values([(dec!(12345), jpy), (dec!(56.78), usd)])
                - Amount::from_values([(dec!(56.780), usd), (dec!(200), eur), (dec!(-13.3), chf)]),
            Amount::from_values([(dec!(12345), jpy), (dec!(-200), eur), (dec!(13.3), chf)])
        );
    }

    #[test]
    fn test_mul() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let eur = ctx.commodities.ensure("EUR");
        let chf = ctx.commodities.ensure("CHF");

        assert_eq!(Amount::zero() * dec!(5), Amount::zero());
        assert_eq!(
            Amount::from_value(dec!(1), jpy) * Decimal::ZERO,
            Amount::zero(),
        );
        assert_eq!(
            Amount::from_value(dec!(123), jpy) * dec!(3),
            Amount::from_value(dec!(369), jpy),
        );
        assert_eq!(
            Amount::from_values([(dec!(10081), jpy), (dec!(200), eur), (dec!(-13.3), chf)])
                * dec!(-0.5),
            Amount::from_values([(dec!(-5040.5), jpy), (dec!(-100.0), eur), (dec!(6.65), chf)]),
        );
    }

    #[test]
    fn test_check_div() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let eur = ctx.commodities.ensure("EUR");
        let chf = ctx.commodities.ensure("CHF");

        assert_eq!(Amount::zero().check_div(dec!(5)).unwrap(), Amount::zero());
        assert_eq!(
            Amount::zero().check_div(dec!(0)).unwrap_err(),
            EvalError::DivideByZero
        );

        assert_eq!(
            Amount::from_value(dec!(50), jpy)
                .check_div(dec!(4))
                .unwrap(),
            Amount::from_value(dec!(12.5), jpy)
        );

        assert_eq!(
            Amount::from_value(Decimal::MAX, jpy)
                .check_div(Decimal::from_i128_with_scale(1, 28))
                .unwrap_err(),
            EvalError::NumberOverflow
        );

        assert_eq!(
            Amount::from_values([(dec!(810), jpy), (dec!(-100.0), eur), (dec!(6.66), chf)])
                .check_div(dec!(3))
                .unwrap(),
            Amount::from_values([
                (dec!(270), jpy),
                (dec!(-33.333333333333333333333333333), eur),
                (dec!(2.22), chf)
            ]),
        );
    }
}
