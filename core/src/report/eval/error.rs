/// Errors specific to expression evaluation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EvalError {
    #[error("operator can't be applied to unmatched types")]
    UnmatchingOperation,
    #[error("unmatching commodities {0} and {1}")]
    UnmatchingCommodities(String, String),
    #[error("cannot divide by zero")]
    DivideByZero,
    #[error("overflow happened")]
    NumberOverflow,
    #[error("expected 0 or amount with commodity")]
    AmountRequired,
    #[error("0 or amount with single commodity expected")]
    PostingAmountRequired,
    #[error("amount with single commodity expected")]
    SingleAmountRequired,
}
