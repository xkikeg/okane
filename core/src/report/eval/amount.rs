use std::{
    collections::{hash_map, HashMap},
    fmt::Display,
    iter::FusedIterator,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use rust_decimal::Decimal;

use crate::report::{commodity::Commodity, context::ReportContext};

use super::{error::EvalError, PostingAmount, SingleAmount};

/// Amount with multiple commodities, or simple zero.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Amount<'ctx> {
    // if values.len == zero, then it'll be completely zero.
    // TODO: Consider optimizing for small number of commodities,
    // as most of the case it needs to be just a few elements.
    values: HashMap<Commodity<'ctx>, Decimal>,
}

impl<'ctx> TryFrom<Amount<'ctx>> for SingleAmount<'ctx> {
    type Error = EvalError;

    fn try_from(value: Amount<'ctx>) -> Result<Self, Self::Error> {
        SingleAmount::try_from(&value)
    }
}

impl<'ctx> TryFrom<Amount<'ctx>> for PostingAmount<'ctx> {
    type Error = EvalError;

    fn try_from(value: Amount<'ctx>) -> Result<Self, Self::Error> {
        PostingAmount::try_from(&value)
    }
}

impl<'ctx> TryFrom<&Amount<'ctx>> for SingleAmount<'ctx> {
    type Error = EvalError;

    fn try_from(value: &Amount<'ctx>) -> Result<Self, Self::Error> {
        let (commodity, value) = value
            .values
            .iter()
            .next()
            .ok_or(EvalError::SingleAmountRequired)?;
        Ok(SingleAmount {
            value: *value,
            commodity: *commodity,
        })
    }
}

impl<'ctx> TryFrom<&Amount<'ctx>> for PostingAmount<'ctx> {
    type Error = EvalError;

    fn try_from(value: &Amount<'ctx>) -> Result<Self, Self::Error> {
        if value.values.len() > 1 {
            Err(EvalError::PostingAmountRequired)
        } else {
            Ok(value
                .values
                .iter()
                .next()
                .map(|(commodity, value)| {
                    PostingAmount::Single(SingleAmount {
                        value: *value,
                        commodity: *commodity,
                    })
                })
                .unwrap_or_default())
        }
    }
}

impl<'ctx> From<PostingAmount<'ctx>> for Amount<'ctx> {
    fn from(value: PostingAmount<'ctx>) -> Self {
        match value {
            PostingAmount::Zero => Amount::zero(),
            PostingAmount::Single(single_amount) => single_amount.into(),
        }
    }
}

impl<'ctx> From<SingleAmount<'ctx>> for Amount<'ctx> {
    fn from(value: SingleAmount<'ctx>) -> Self {
        Amount::from_value(value.value, value.commodity)
    }
}

impl<'ctx> Amount<'ctx> {
    /// Creates an [`Amount`] with zero value.
    #[inline(always)]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Creates an [`Amount`] with single value and commodity.
    pub fn from_value(amount: Decimal, commodity: Commodity<'ctx>) -> Self {
        Self::zero() + SingleAmount::from_value(amount, commodity)
    }

    /// Creates an [`Amount`] from a set of values.
    pub fn from_values<T>(values: T) -> Self
    where
        T: IntoIterator<Item = (Decimal, Commodity<'ctx>)>,
    {
        let mut ret = Amount::zero();
        for (value, commodity) in values.into_iter() {
            ret += SingleAmount::from_value(value, commodity);
        }
        ret
    }

    /// Takes out the instance and returns map from commodity to its value.
    pub fn into_values(self) -> HashMap<Commodity<'ctx>, Decimal> {
        self.values
    }

    /// Returns iterator over its amount.
    pub fn iter(&self) -> impl Iterator<Item = SingleAmount<'ctx>> + '_ {
        AmountIter(self.values.iter())
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

    /// Removes zero values, useful when callers doesn't care zero value.
    /// However, if caller must distinguish `0` and `0 commodity`,
    /// caller must not use this method.
    pub fn remove_zero_entries(&mut self) {
        self.values.retain(|_, v| !v.is_zero());
    }

    /// Replace the amount of the particular commodity, and returns the previous amount for the commodity.
    /// E.g. (100 USD + 100 EUR).set_partial(200, USD) returns 100.
    /// Note this method removes the given commodity if value is zero,
    /// so only meant for [`Balance`].
    pub(crate) fn set_partial(&mut self, amount: SingleAmount<'ctx>) -> SingleAmount<'ctx> {
        let value = if amount.value.is_zero() {
            self.values.remove(&amount.commodity)
        } else {
            self.values.insert(amount.commodity, amount.value)
        }
        .unwrap_or_default();
        SingleAmount {
            value,
            commodity: amount.commodity,
        }
    }

    /// Returns the amount of the particular commodity.
    fn get_part(&self, commodity: Commodity<'ctx>) -> Decimal {
        self.values.get(&commodity).copied().unwrap_or_default()
    }

    /// Returns pair of commodity amount, if the amount contains exactly 2 commodities.
    /// Otherwise returns None.
    pub fn maybe_pair(&self) -> Option<(SingleAmount<'ctx>, SingleAmount<'ctx>)> {
        if self.values.len() != 2 {
            return None;
        }
        let ((c1, v1), (c2, v2)) = self.values.iter().zip(self.values.iter().skip(1)).next()?;
        Some((
            SingleAmount::from_value(*v1, *c1),
            SingleAmount::from_value(*v2, *c2),
        ))
    }

    /// Rounds the given Amount and returns the new instance.
    pub fn round(mut self, ctx: &ReportContext) -> Self {
        self.round_mut(ctx);
        self
    }

    /// Rounds the Amount in-place with the given context provided precision.
    pub fn round_mut(&mut self, ctx: &ReportContext) {
        for (k, v) in self.values.iter_mut() {
            match ctx.commodities.get_decimal_point(*k) {
                None => (),
                Some(dp) => {
                    let updated = v.round_dp_with_strategy(
                        dp,
                        rust_decimal::RoundingStrategy::MidpointNearestEven,
                    );
                    *v = updated;
                }
            }
        }
    }

    /// Creates negated instance.
    pub fn negate(mut self) -> Self {
        for (_, v) in self.values.iter_mut() {
            v.set_sign_positive(!v.is_sign_positive())
        }
        self
    }

    /// Run division with error checking.
    pub fn check_div(mut self, rhs: Decimal) -> Result<Self, EvalError> {
        if rhs.is_zero() {
            return Err(EvalError::DivideByZero);
        }
        for (_, v) in self.values.iter_mut() {
            *v = v.checked_div(rhs).ok_or(EvalError::NumberOverflow)?;
        }
        Ok(self)
    }

    /// Checks if the amount is matching with the given [`PostingAmount`] balance,
    /// Returns the diff (expected - actual), or None if those are consistent.
    ///
    /// Consistent means
    ///
    /// *   If the given balance is zero, then the amount must be zero.
    /// *   If the given balance is a value with commodity,
    ///     then the amount should be equal to given value only on the commodity.
    pub(crate) fn assert_balance(&self, expected: &PostingAmount<'ctx>) -> Self {
        match expected {
            PostingAmount::Zero => {
                if self.is_zero() {
                    Self::zero()
                } else {
                    -self.clone()
                }
            }
            PostingAmount::Single(single) => {
                let diff = single.value - self.get_part(single.commodity);
                if diff.is_zero() {
                    Self::zero()
                } else {
                    Self::from_value(diff, single.commodity)
                }
            }
        }
    }
}

#[derive(Debug)]
struct AmountIter<'a, 'ctx>(hash_map::Iter<'a, Commodity<'ctx>, Decimal>);

impl<'ctx> Iterator for AmountIter<'_, 'ctx> {
    type Item = SingleAmount<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(c, v)| SingleAmount::from_value(*v, *c))
    }
}

impl FusedIterator for AmountIter<'_, '_> {}

#[derive(Debug)]
struct InlinePrintAmount<'a, 'ctx>(&'a Amount<'ctx>);

impl Display for InlinePrintAmount<'_, '_> {
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

impl Neg for Amount<'_> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

impl Add for Amount<'_> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Amount<'_> {
    fn add_assign(&mut self, rhs: Self) {
        for (c, v2) in rhs.values {
            let mut v1 = self.values.entry(c).or_insert(Decimal::ZERO);
            v1 += v2;
            // we should retain the value even if zero,
            // as (0 USD + 0 EUR) are different from 0 or (0 USD + 0 USD).
        }
    }
}

impl<'ctx> Add<SingleAmount<'ctx>> for Amount<'ctx> {
    type Output = Amount<'ctx>;

    fn add(mut self, rhs: SingleAmount<'ctx>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'ctx> AddAssign<SingleAmount<'ctx>> for Amount<'ctx> {
    fn add_assign(&mut self, rhs: SingleAmount<'ctx>) {
        let curr = self.values.entry(rhs.commodity).or_default();
        *curr += rhs.value;
    }
}

impl<'ctx> AddAssign<PostingAmount<'ctx>> for Amount<'ctx> {
    fn add_assign(&mut self, rhs: PostingAmount<'ctx>) {
        match rhs {
            PostingAmount::Zero => (),
            PostingAmount::Single(single) => *self += single,
        }
    }
}

impl Sub for Amount<'_> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for Amount<'_> {
    fn sub_assign(&mut self, rhs: Self) {
        for (c, v2) in rhs.values {
            let mut v1 = self.values.entry(c).or_insert(Decimal::ZERO);
            v1 -= v2;
        }
    }
}

impl Mul<Decimal> for Amount<'_> {
    type Output = Self;

    fn mul(mut self, rhs: Decimal) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<Decimal> for Amount<'_> {
    fn mul_assign(&mut self, rhs: Decimal) {
        for (_, mut v) in self.values.iter_mut() {
            v *= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::{report::ReportContext, syntax::pretty_decimal::PrettyDecimal};

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
    fn test_from_values() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let chf = ctx.commodities.ensure("CHF");

        let amount = Amount::from_values([(dec!(10), jpy), (dec!(1), chf)]);
        assert_eq!(
            amount.into_values(),
            hashmap! {jpy => dec!(10), chf => dec!(1)},
        );

        let amount = Amount::from_values([(dec!(10), jpy), (dec!(1), jpy)]);
        assert_eq!(amount.into_values(), hashmap! {jpy => dec!(11)});

        let amount = Amount::from_values([(dec!(10), jpy), (dec!(-10), jpy)]);
        assert_eq!(amount.into_values(), hashmap! {jpy => dec!(0)});
    }

    #[test]
    fn test_is_absolute_zero() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        assert!(Amount::default().is_absolute_zero());
        assert!(!Amount::from_value(dec!(0), jpy).is_absolute_zero());

        let mut amount = Amount::from_values([(dec!(0), jpy), (dec!(0), usd)]);
        assert!(!amount.is_absolute_zero(), "{}", amount.as_inline_display());

        amount.remove_zero_entries();
        assert!(amount.is_absolute_zero(), "{}", amount.as_inline_display());
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
    fn test_add_amount() {
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
            Amount::from_values([
                (dec!(123.00), jpy),
                (dec!(456.0), usd),
                (dec!(7.89), eur),
                (dec!(0), chf), // 0 CHF retained
            ]),
            Amount::from_value(dec!(123.45), jpy)
                + Amount::from_value(dec!(-0.45), jpy)
                + Amount::from_value(dec!(456), usd)
                + Amount::from_value(dec!(0.0), usd)
                + -Amount::from_value(dec!(100), chf)
                + Amount::from_value(dec!(7.89), eur)
                + Amount::from_value(dec!(100), chf),
        );

        assert_eq!(
            Amount::from_values([(dec!(0), jpy), (dec!(0), usd), (dec!(0), chf)]),
            Amount::from_values([(dec!(1), jpy), (dec!(2), usd), (dec!(3), chf)])
                + Amount::from_values([(dec!(-1), jpy), (dec!(-2), usd), (dec!(-3), chf)])
        );
    }

    #[test]
    fn test_add_single_amount() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        let amount = Amount::zero() + SingleAmount::from_value(dec!(0), usd);
        assert_eq!(amount, Amount::from_value(dec!(0), usd));

        assert_eq!(
            Amount::zero() + SingleAmount::from_value(dec!(1), jpy),
            Amount::from_value(dec!(1), jpy),
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
            Amount::from_values([
                (dec!(12345), jpy),
                (dec!(-200), eur),
                (dec!(13.3), chf),
                (dec!(0), usd)
            ]),
            Amount::from_values([(dec!(12345), jpy), (dec!(56.78), usd)])
                - Amount::from_values([(dec!(56.780), usd), (dec!(200), eur), (dec!(-13.3), chf),]),
        );
    }

    fn eps() -> Decimal {
        Decimal::try_from_i128_with_scale(1, 28).unwrap()
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
            Amount::from_value(dec!(0), jpy),
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
        assert_eq!(
            Amount::from_value(eps(), jpy) * eps(),
            Amount::from_value(dec!(0), jpy)
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
                .check_div(eps())
                .unwrap_err(),
            EvalError::NumberOverflow
        );

        assert_eq!(
            Amount::from_value(eps(), jpy)
                .check_div(Decimal::MAX)
                .unwrap(),
            Amount::from_value(dec!(0), jpy)
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

    #[test]
    fn test_round() {
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

        assert_eq!(Amount::zero(), Amount::zero().round(&ctx));

        assert_eq!(
            Amount::from_values([(dec!(812), jpy), (dec!(-100.00), eur), (dec!(6.660), chf)]),
            Amount::from_values([(dec!(812), jpy), (dec!(-100.0), eur), (dec!(6.66), chf)])
                .round(&ctx),
        );

        assert_eq!(
            Amount::from_values([(dec!(812), jpy), (dec!(-100.02), eur), (dec!(6.666), chf)]),
            Amount::from_values([
                (dec!(812.5), jpy),
                (dec!(-100.015), eur),
                (dec!(6.6665), chf)
            ])
            .round(&ctx),
        );
    }
}
