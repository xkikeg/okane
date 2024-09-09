use rust_decimal::Decimal;

/// Represents simple owned Amount.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OwnedAmount {
    pub value: Decimal,
    pub commodity: String,
}

impl OwnedAmount {
    /// Returns `true` if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    /// Returns `true` if the amount is positive.
    pub fn is_sign_positive(&self) -> bool {
        self.value.is_sign_positive()
    }

    /// Returns `true` if the amount is negative.
    pub fn is_sign_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
}

/// # Examples
///
/// ```
/// # use rust_decimal_macros::dec;
/// let x = okane::import::amount::OwnedAmount{
///     value: dec!(-5),
///     commodity: "JPY".to_string(),
/// };
/// let y = -x.clone();
/// assert_eq!(y.value, dec!(5));
/// assert_eq!(y.commodity, "JPY");
/// ```
impl std::ops::Neg for OwnedAmount {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            commodity: self.commodity,
        }
    }
}
