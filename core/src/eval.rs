//! eval module contains functions for Ledger file evaluation.

use std::path::PathBuf;

use crate::repl::{
    self,
    parser::{parse_ledger, ParseLedgerError},
    LedgerEntry,
};

/// Loads the given path and returns the iterator.
pub fn load(path: &std::path::Path) -> Result<Vec<repl::LedgerEntry>, LoadError> {
    let mut r = Vec::new();
    load_impl(path, &mut r)?;
    Ok(r)
}

fn load_impl(path: &std::path::Path, ret: &mut Vec<LedgerEntry>) -> Result<(), LoadError> {
    let content = std::fs::read_to_string(path)?;
    let vs = parse_ledger(&content)?;
    for elem in vs.into_iter() {
        match elem {
            repl::LedgerEntry::Include(p) => {
                let include_path: PathBuf = p.0.into();
                let target = path
                    .parent()
                    .ok_or_else(|| LoadError::IncludePath(path.to_owned()))?
                    .join(include_path);
                load_impl(target.as_path(), ret)?
            }
            _ => ret.push(elem),
        }
    }
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse file {0}")]
    Parse(#[from] ParseLedgerError),
    #[error("unexpected include path {0}")]
    IncludePath(PathBuf),
}
