use crate::report::{
    commodity::{CommodityTag, OwnedCommodity},
    context::ReportContext,
};

/// Errors specific to expression evaluation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EvalError<'ctx> {
    #[error("the operator can't be applied to unmatching types")]
    UnmatchingOperation,
    #[error("unmatching commodities: {} and {}", .0.as_index(), .1.as_index())]
    UnmatchingCommodities(CommodityTag<'ctx>, CommodityTag<'ctx>),
    #[error("unknown commodity {0}")]
    UnknownCommodity(OwnedCommodity),
    #[error("cannot divide by zero")]
    DivideByZero,
    #[error("overflow happened")]
    NumberOverflow,
    #[error("expected 0 or an amount with commodities")]
    AmountRequired,
    #[error("expected 0 or an amount with the single commodity")]
    PostingAmountRequired,
    #[error("expected an amount with the single commodity")]
    SingleAmountRequired,
}

/// Owned version of [`EvalError`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OwnedEvalError {
    #[error("the operator can't be applied to unmatching types")]
    UnmatchingOperation,
    #[error("unmatching commodities: {0} and {1}")]
    UnmatchingCommodities(OwnedCommodity, OwnedCommodity),
    #[error("unknown commodity {0}")]
    UnknownCommodity(OwnedCommodity),
    #[error("cannot divide by zero")]
    DivideByZero,
    #[error("overflow happened")]
    NumberOverflow,
    #[error("expected 0 or an amount with commodities")]
    AmountRequired,
    #[error("expected 0 or an amount with the single commodity")]
    PostingAmountRequired,
    #[error("expected an amount with the single commodity")]
    SingleAmountRequired,
}

impl<'ctx> EvalError<'ctx> {
    /// Converts the self into [`OwnedEvalError`] so to be embedded in the other error #[from]
    pub(crate) fn into_owned(self, ctx: &ReportContext<'ctx>) -> OwnedEvalError {
        match self {
            EvalError::UnmatchingOperation => OwnedEvalError::UnmatchingOperation,
            EvalError::UnmatchingCommodities(c1, c2) => OwnedEvalError::UnmatchingCommodities(
                c1.to_owned_lossy(&ctx.commodities),
                c2.to_owned_lossy(&ctx.commodities),
            ),
            EvalError::UnknownCommodity(c1) => OwnedEvalError::UnknownCommodity(c1),
            EvalError::DivideByZero => OwnedEvalError::DivideByZero,
            EvalError::NumberOverflow => OwnedEvalError::NumberOverflow,
            EvalError::AmountRequired => OwnedEvalError::AmountRequired,
            EvalError::PostingAmountRequired => OwnedEvalError::PostingAmountRequired,
            EvalError::SingleAmountRequired => OwnedEvalError::SingleAmountRequired,
        }
    }
}
