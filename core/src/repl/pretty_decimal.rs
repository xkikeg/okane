use std::{convert::TryInto, fmt::Display, str::FromStr};

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
    #[error("unexpected char {0} at {0}")]
    UnexpectedChar(u8, usize),
    #[error("comma required at {0}")]
    CommaRequired(usize),
    #[error("unexpressible decimal {0}")]
    InvalidDecimal(#[from] rust_decimal::Error),
}

impl PrettyDecimal {
    /// Constructs a new instance with [Format].
    pub fn with_format(value: Decimal, format: Option<Format>) -> Self {
        Self { format, value }
    }

    /// Constructs unformatted PrettyDecimal.
    #[inline]
    pub fn unformatted(value: Decimal) -> Self {
        Self::with_format(value, None)
    }

    /// Constructs plain PrettyDecimal.
    #[inline]
    pub fn plain(value: Decimal) -> Self {
        Self::with_format(value, Some(Format::Plain))
    }

    /// Constructs comma3 PrettyDecimal.
    #[inline]
    pub fn comma3dot(value: Decimal) -> Self {
        Self::with_format(value, Some(Format::Comma3Dot))
    }

    /// Returns the current scale.
    pub fn scale(&self) -> u32 {
        self.value.scale()
    }

    /// Rescale the underlying value.
    pub fn rescale(&mut self, scale: u32) {
        self.value.rescale(scale)
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
                    return Err(Error::UnexpectedChar(c, i));
                }
            }
        }
        let value = Decimal::try_from_i128_with_scale(sign * mantissa, scale.unwrap_or(0))?;
        Ok(Self { format, value })
    }
}

impl Display for PrettyDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            Some(Format::Plain) | None => self.value.fmt(f),
            Some(Format::Comma3Dot) => {
                if self.value.is_sign_negative() {
                    write!(f, "-")?;
                }
                let mantissa = self.value.abs().mantissa().to_string();
                let scale: usize = self
                    .value
                    .scale()
                    .try_into()
                    .expect("32-bit or larger bit only");
                let mut remainder = mantissa.as_str();
                // Here we assume mantissa is all ASCII (given it's [0-9.]+)
                let mut initial_integer = true;
                // caluclate the first comma position out of the integral portion digits.
                let mut comma_pos = (mantissa.len() - scale) % 3;
                if comma_pos == 0 {
                    comma_pos = 3;
                }
                while remainder.len() > scale {
                    if !initial_integer {
                        write!(f, ",")?;
                    }
                    let section;
                    (section, remainder) = remainder.split_at(comma_pos);
                    write!(f, "{}", section)?;
                    comma_pos = 3;
                    initial_integer = false;
                }
                if initial_integer {
                    write!(f, "0")?;
                }
                if !remainder.is_empty() {
                    write!(f, ".{}", remainder)?;
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

        assert_eq!(
            "1.234000",
            PrettyDecimal::comma3dot(dec!(1.234000)).to_string()
        );

        assert_eq!("123.4", PrettyDecimal::comma3dot(dec!(123.4)).to_string());

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
