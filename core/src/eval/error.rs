use std::path::PathBuf;

use crate::repl;

use super::amounts;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("operator can't be applied to unmatched types")]
    UnmatchingOperation,
    #[error("cannot divide by zero")]
    DivideByZero,
    #[error("overflow happened")]
    NumberOverflow,
}

#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    /// Failed to eval the given expression.
    #[error("failed to eval the expression")]
    EvalFailure(#[from] EvalError),
    #[error("posting amount must be resolved as a simple value with commodity or zero")]
    ComplexPostingAmount,
    #[error("transaction cannot have multiple postings without amount: {0} {1}")]
    UndeduciblePostingAmount(usize, usize),
    #[error("transaction cannot have unbalanced postings: {0}")]
    UnbalancedPostings(String),
}

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse file {0}")]
    Parse(#[from] repl::parser::ParseLedgerError),
    #[error("unexpected include path {0}")]
    IncludePath(PathBuf),
}
