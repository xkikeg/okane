//! Defines error in report functions.

use crate::load;

/// Error arised in report APIs.
#[derive(thiserror::Error, Debug)]
pub enum ReportError {
    #[error("failed to load the given file")]
    Load(#[from] load::LoadError),
}
