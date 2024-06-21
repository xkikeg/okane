/// Errors specific to expression evaluation.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum EvalError {
    #[error("operator can't be applied to unmatched types")]
    UnmatchingOperation,
    #[error("cannot divide by zero")]
    DivideByZero,
    #[error("overflow happened")]
    NumberOverflow,
    #[error("expected 0 or amount with commodity")]
    CommodityAmountRequired,
}
