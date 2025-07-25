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
