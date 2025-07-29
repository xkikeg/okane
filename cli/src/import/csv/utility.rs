use rust_decimal::Decimal;

use okane_core::syntax;

use crate::import::ImportError;

pub(super) fn str_to_comma_decimal(input: &str) -> Result<Option<Decimal>, ImportError> {
    if input.is_empty() {
        return Ok(None);
    }
    let a: syntax::expr::Amount = input
        .try_into()
        .map_err(|e| ImportError::Other(format!("failed to parse comma decimal: {}", e)))?;
    Ok(Some(a.value.value))
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn parses_regular_decimal() {
        assert_eq!(
            Some(dec!(1234.56)),
            str_to_comma_decimal("1234.56").unwrap(),
        );
    }

    #[test]
    fn parses_comma_decimal() {
        assert_eq!(
            Some(dec!(1234.56)),
            str_to_comma_decimal("$1,234.56").unwrap(),
        );
    }

    #[test]
    fn parses_empty() {
        assert_eq!(None, str_to_comma_decimal("").unwrap());
    }

    #[test]
    fn parses_invalid() {
        assert!(matches!(
            str_to_comma_decimal("invalid"),
            Err(ImportError::Other(_))
        ));
    }
}
