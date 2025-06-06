//! Module to define [PrettyDecimal], [Decimal] with formatting.

use std::{convert::TryInto, fmt::Display, ops::Neg, str::FromStr};

use bounded_static::{IntoBoundedStatic, ToBoundedStatic, ToStatic};
use rust_decimal::Decimal;

/// Decimal formatting type for pretty-printing.
#[derive(Debug, PartialEq, Eq, Clone, Copy, ToStatic)]
#[non_exhaustive]
pub enum Format {
    /// Decimal without no formatting, such as
    /// `1234` or `1234.5`.
    Plain,
    /// Use `,` on every thousands, `.` for the decimal point.
    Comma3Dot,
}

/// Decimal with the original format information encoded.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
#[non_exhaustive] // Don't want to construct directly.
pub struct PrettyDecimal {
    /// Format of the decimal, None means there's no associated information.
    pub format: Option<Format>,
    pub value: Decimal,
}

impl Neg for PrettyDecimal {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.set_sign_positive(!self.value.is_sign_positive());
        self
    }
}

impl ToBoundedStatic for PrettyDecimal {
    type Static = Self;

    fn to_static(&self) -> <Self as ToBoundedStatic>::Static {
        self.clone()
    }
}

impl IntoBoundedStatic for PrettyDecimal {
    type Static = Self;

    fn into_static(self) -> <Self as IntoBoundedStatic>::Static {
        self
    }
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum Error {
    #[error("unexpected char {0} at {1}")]
    UnexpectedChar(String, usize),
    #[error("comma required at {0}")]
    CommaRequired(usize),
    #[error("unexpressible decimal {0}")]
    InvalidDecimal(#[from] rust_decimal::Error),
}

impl PrettyDecimal {
    /// Constructs unformatted PrettyDecimal.
    #[inline]
    pub const fn unformatted(value: Decimal) -> Self {
        Self {
            format: None,
            value,
        }
    }

    /// Constructs plain PrettyDecimal.
    #[inline]
    pub const fn plain(value: Decimal) -> Self {
        Self {
            format: Some(Format::Plain),
            value,
        }
    }

    /// Constructs comma3 PrettyDecimal.
    #[inline]
    pub const fn comma3dot(value: Decimal) -> Self {
        Self {
            format: Some(Format::Comma3Dot),
            value,
        }
    }

    /// Returns the current scale.
    pub const fn scale(&self) -> u32 {
        self.value.scale()
    }

    /// Rescale the underlying value.
    pub fn rescale(&mut self, scale: u32) {
        self.value.rescale(scale)
    }

    /// Sets the sign positive.
    pub fn set_sign_positive(&mut self, positive: bool) {
        self.value.set_sign_positive(positive)
    }

    /// Returns `true` if the value is positive.
    pub const fn is_sign_positive(&self) -> bool {
        self.value.is_sign_positive()
    }
}

impl From<PrettyDecimal> for Decimal {
    #[inline]
    fn from(value: PrettyDecimal) -> Self {
        value.value
    }
}

impl FromStr for PrettyDecimal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Only ASCII chars supported, use bytes.
        let mut comma_pos = None;
        let mut format = None;
        let mut mantissa: i128 = 0;
        let mut scale: Option<u32> = None;
        let mut prefix_len = 0;
        let mut sign = 1;
        let aligned_comma = |offset, cp, pos| match (cp, pos) {
            (None, _) if pos > offset && pos <= 3 + offset => true,
            _ if cp == Some(pos) => true,
            _ => false,
        };
        for (i, c) in s.bytes().enumerate() {
            match (comma_pos, i, c) {
                (_, 0, b'-') => {
                    prefix_len = 1;
                    sign = -1;
                }
                (_, _, b',') if aligned_comma(prefix_len, comma_pos, i) => {
                    format = Some(Format::Comma3Dot);
                    comma_pos = Some(i + 4);
                }
                (_, _, b'.') if comma_pos.is_none() || comma_pos == Some(i) => {
                    scale = Some(0);
                    comma_pos = None;
                }
                (Some(cp), _, _) if cp == i => {
                    return Err(Error::CommaRequired(i));
                }
                _ if c.is_ascii_digit() => {
                    if scale.is_none() && format.is_none() && i >= 3 + prefix_len {
                        format = Some(Format::Plain);
                    }
                    mantissa = mantissa * 10 + (c as u32 - '0' as u32) as i128;
                    scale = scale.map(|x| x + 1);
                }
                _ => {
                    return Err(Error::UnexpectedChar(try_find_char(s, i, c), i));
                }
            }
        }
        let value = Decimal::try_from_i128_with_scale(sign * mantissa, scale.unwrap_or(0))?;
        Ok(Self { format, value })
    }
}

// Find the char at i. Note it returns String instead of char for complicated situations.
fn try_find_char(s: &str, i: usize, chr: u8) -> String {
    let begin = (0..=i).rev().find(|j| s.is_char_boundary(*j)).unwrap_or(0);
    let end = (i + 1..s.len())
        .find(|j| s.is_char_boundary(*j))
        .unwrap_or(s.len());
    s.get(begin..end)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{:?}", chr))
}

impl Display for PrettyDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;

        match self.format {
            Some(Format::Plain) | None => self.value.fmt(f),
            Some(Format::Comma3Dot) => {
                if self.value.abs() < Decimal::new(1000, 0) {
                    // no comma needed.
                    return self.value.fmt(f);
                }
                if self.value.is_sign_negative() {
                    write!(f, "-")?;
                }
                let mut buf = itoa::Buffer::new();
                let mantissa = buf.format(self.value.abs().mantissa());
                let scale: usize = self
                    .value
                    .scale()
                    .try_into()
                    .expect("32-bit or larger bit system only");
                let mut remainder = mantissa;
                // Here we assume mantissa is all ASCII (given it's [0-9.]+)
                let mut initial_integer = true;
                // caluclate the first comma position out of the integral portion digits.
                let mut comma_pos = (mantissa.len() - scale) % 3;
                if comma_pos == 0 {
                    comma_pos = 3;
                }
                while remainder.len() > scale {
                    if !initial_integer {
                        f.write_char(',')?;
                    }
                    let section;
                    (section, remainder) = remainder.split_at(comma_pos);
                    f.write_str(section)?;
                    comma_pos = 3;
                    initial_integer = false;
                }
                if !remainder.is_empty() {
                    f.write_char('.')?;
                    f.write_str(remainder)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn from_str_unformatted() {
        // If the number is below 1000, we can't tell if the number is plain or comma3dot.
        // Thus we declare them as unformatted instead of plain.
        assert_eq!(Ok(PrettyDecimal::unformatted(dec!(1))), "1".parse());
        assert_eq!(Ok(PrettyDecimal::unformatted(dec!(-1))), "-1".parse());

        assert_eq!(Ok(PrettyDecimal::unformatted(dec!(12))), "12".parse());
        assert_eq!(Ok(PrettyDecimal::unformatted(dec!(-12))), "-12".parse());

        assert_eq!(Ok(PrettyDecimal::unformatted(dec!(123))), "123".parse());
        assert_eq!(Ok(PrettyDecimal::unformatted(dec!(-123))), "-123".parse());

        assert_eq!(
            Ok(PrettyDecimal::unformatted(dec!(0.123450))),
            "0.123450".parse()
        );
    }

    #[test]
    fn from_str_plain() {
        assert_eq!(Ok(PrettyDecimal::plain(dec!(1234))), "1234".parse());
        assert_eq!(Ok(PrettyDecimal::plain(dec!(-1234))), "-1234".parse());

        assert_eq!(Ok(PrettyDecimal::plain(dec!(1234567))), "1234567".parse());
        assert_eq!(Ok(PrettyDecimal::plain(dec!(-1234567))), "-1234567".parse());

        assert_eq!(Ok(PrettyDecimal::plain(dec!(1234.567))), "1234.567".parse());
        assert_eq!(
            Ok(PrettyDecimal::plain(dec!(-1234.567))),
            "-1234.567".parse()
        );
    }

    #[test]
    fn from_str_comma() {
        assert_eq!(Ok(PrettyDecimal::comma3dot(dec!(1234))), "1,234".parse());
        assert_eq!(Ok(PrettyDecimal::comma3dot(dec!(-1234))), "-1,234".parse());

        assert_eq!(Ok(PrettyDecimal::comma3dot(dec!(12345))), "12,345".parse());
        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(-12345))),
            "-12,345".parse()
        );

        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(123456))),
            "123,456".parse()
        );
        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(-123456))),
            "-123,456".parse()
        );

        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(1234567))),
            "1,234,567".parse()
        );
        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(-1234567))),
            "-1,234,567".parse()
        );

        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(1234.567))),
            "1,234.567".parse()
        );
        assert_eq!(
            Ok(PrettyDecimal::comma3dot(dec!(-1234.567))),
            "-1,234.567".parse()
        );
    }

    #[test]
    fn display_plain() {
        assert_eq!("1.234000", PrettyDecimal::plain(dec!(1.234000)).to_string());
    }

    #[test]
    fn display_comma3_dot() {
        assert_eq!("123", PrettyDecimal::comma3dot(dec!(123)).to_string());

        assert_eq!("-1,234", PrettyDecimal::comma3dot(dec!(-1234)).to_string());

        assert_eq!("0", PrettyDecimal::comma3dot(dec!(0)).to_string());

        assert_eq!("0.1200", PrettyDecimal::comma3dot(dec!(0.1200)).to_string());

        assert_eq!("0.0012", PrettyDecimal::comma3dot(dec!(0.0012)).to_string());

        assert_eq!(
            // 12345678901234567890
            "0.000000000000000000001",
            PrettyDecimal::comma3dot(Decimal::new(1, 21)).to_string()
        );

        assert_eq!(
            "1.234000",
            PrettyDecimal::comma3dot(dec!(1.234000)).to_string()
        );

        assert_eq!("123.4", PrettyDecimal::comma3dot(dec!(123.4)).to_string());
        assert_eq!(
            "999.9999",
            PrettyDecimal::comma3dot(dec!(999.9999)).to_string()
        );
        assert_eq!(
            "-999.9999",
            PrettyDecimal::comma3dot(dec!(-999.9999)).to_string()
        );
        assert_eq!("1,000", PrettyDecimal::comma3dot(dec!(1000)).to_string());
        assert_eq!("-1,000", PrettyDecimal::comma3dot(dec!(-1000)).to_string());

        assert_eq!(
            "1,234,567.890120",
            PrettyDecimal::comma3dot(dec!(1234567.890120)).to_string()
        );
    }

    #[test]
    fn scale_returns_correct_number() {
        assert_eq!(0, PrettyDecimal::comma3dot(dec!(1230)).scale());
        assert_eq!(1, PrettyDecimal::comma3dot(dec!(1230.4)).scale());
    }
}
