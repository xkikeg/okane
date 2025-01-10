//! Defines extra amount types useful for import.

use rust_decimal::Decimal;

/// Represents simple owned Amount.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OwnedAmount {
    pub value: Decimal,
    pub commodity: String,
}

impl std::ops::Neg for OwnedAmount {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            commodity: self.commodity,
        }
    }
}

/// AmountRef unifies [OwnedAmount] reference and [BorrowedAmount].
pub trait AmountRef<'a> {
    fn into_borrowed(self) -> BorrowedAmount<'a>;
}

impl<'a> AmountRef<'a> for &'a OwnedAmount {
    fn into_borrowed(self) -> BorrowedAmount<'a> {
        BorrowedAmount {
            value: self.value,
            commodity: &self.commodity,
        }
    }
}

/// Represents a reference to [OwnedAmount].
/// it's actually pretty close to [okane_core::syntax::expr::Amount],
/// but without any formatting nor Cow.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BorrowedAmount<'a> {
    pub value: Decimal,
    pub commodity: &'a str,
}

impl<'a> AmountRef<'a> for BorrowedAmount<'a> {
    fn into_borrowed(self) -> BorrowedAmount<'a> {
        self
    }
}

impl std::ops::Neg for BorrowedAmount<'_> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            commodity: self.commodity,
        }
    }
}
