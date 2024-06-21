//! Defines error in report functions.

use crate::load;

use super::book_keeping;

/// Error arised in report APIs.
#[derive(thiserror::Error, Debug)]
pub enum ReportError {
    #[error("failed to load the given file")]
    Load(#[from] load::LoadError),
    #[error("failed to process the given transaction")]
    BookKeep(#[from] book_keeping::BookKeepError),
}
