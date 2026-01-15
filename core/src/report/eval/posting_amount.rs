use std::ops::Neg;

#[cfg(test)]
use crate::report::ReportContext;

use super::{error::EvalError, single_amount::SingleAmount};

/// Amount with only one commodity, or total zero.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub(crate) enum PostingAmount<'ctx> {
    #[default]
    Zero,
    Single(SingleAmount<'ctx>),
}

impl<'ctx> TryFrom<PostingAmount<'ctx>> for SingleAmount<'ctx> {
    type Error = EvalError<'ctx>;

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

impl Neg for PostingAmount<'_> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            PostingAmount::Zero => PostingAmount::Zero,
            PostingAmount::Single(amount) => PostingAmount::Single(-amount),
        }
    }
}

impl<'ctx> PostingAmount<'ctx> {
    /// Returns absolute zero.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Adds the amount with keeping commodity single.
    pub fn check_add(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
        match (self, rhs) {
            (PostingAmount::Zero, _) => Ok(rhs),
            (_, PostingAmount::Zero) => Ok(self),
            (PostingAmount::Single(lhs), PostingAmount::Single(rhs)) => {
                lhs.check_add(rhs).map(Self::Single)
            }
        }
    }

    /// Subtracts the amount with keeping the commodity single.
    pub fn check_sub(self, rhs: Self) -> Result<Self, EvalError<'ctx>> {
        self.check_add(-rhs)
    }
}

#[cfg(test)]
struct PostingAmountDisplay<'a, 'ctx>(&'a PostingAmount<'ctx>, &'a ReportContext<'ctx>);

#[cfg(test)]
impl std::fmt::Display for PostingAmountDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        match self.0 {
            PostingAmount::Zero => f.write_char('0'),
            PostingAmount::Single(single) => single.as_display(self.1).fmt(f),
        }
    }
}
#[cfg(test)]
impl<'ctx> PostingAmount<'ctx> {
    /// Returns an instance which can be displayed.
    pub fn as_display<'a>(&'a self, ctx: &'a ReportContext<'ctx>) -> impl std::fmt::Display + 'a
    where
        'a: 'ctx,
    {
        PostingAmountDisplay(self, ctx)
    }

    /// Constructs an instance with single commodity.
    pub(crate) fn from_value(
        commodity: crate::report::commodity::CommodityTag<'ctx>,
        value: rust_decimal::Decimal,
    ) -> Self {
        PostingAmount::Single(SingleAmount::from_value(commodity, value))
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

        let jpy = ctx.commodities.insert("JPY").unwrap();

        assert_eq!(PostingAmount::Zero, -PostingAmount::zero());

        assert_eq!(
            PostingAmount::from_value(jpy, dec!(5)),
            -PostingAmount::from_value(jpy, dec!(-5)),
        );
    }

    #[test]
    fn check_add() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();

        assert_eq!(
            PostingAmount::from_value(jpy, dec!(5)),
            PostingAmount::from_value(jpy, dec!(5))
                .check_add(PostingAmount::zero())
                .unwrap(),
        );

        assert_eq!(
            PostingAmount::from_value(jpy, dec!(5)),
            PostingAmount::zero()
                .check_add(PostingAmount::from_value(jpy, dec!(5)))
                .unwrap(),
        );
    }

    #[test]
    fn display() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);

        let jpy = ctx.commodities.insert("JPY").unwrap();

        assert_eq!("0", format!("{}", PostingAmount::zero().as_display(&ctx)));
        assert_eq!(
            "1.23 JPY",
            format!(
                "{}",
                PostingAmount::from_value(jpy, dec!(1.23)).as_display(&ctx)
            )
        );
    }
}
