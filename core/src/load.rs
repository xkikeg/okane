//! Module `load` contains the functions useful for loading Ledger file,
//! recursively resolving the `include` directives.

use std::path::{Path, PathBuf};

use crate::{parse, repl};

/// Error caused by `load_*` functions.
#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse file {0}")]
    Parse(#[from] parse::ParseError),
    #[error("unexpected include path {0}, maybe filesystem root is passed")]
    IncludePath(PathBuf),
}

/// Loads `repl::LedgerEntry` and invoke callback on every entry,
/// recursively resolving `include` directives.
pub fn load_repl<T, E>(path: &Path, mut callback: T) -> Result<(), E>
where
    T: FnMut(&Path, &parse::ParsedLedgerEntry<'_>) -> Result<(), E>,
    E: std::error::Error + From<LoadError>,
{
    load_repl_impl(path, &mut callback)
}

fn load_repl_impl<T, E>(path: &Path, callback: &mut T) -> Result<(), E>
where
    T: FnMut(&Path, &parse::ParsedLedgerEntry<'_>) -> Result<(), E>,
    E: std::error::Error + From<LoadError>,
{
    let content = std::fs::read_to_string(path).map_err(LoadError::IO)?;
    for entry in parse::parse_ledger(&content) {
        match entry.map_err(LoadError::Parse)? {
            repl::LedgerEntry::Include(p) => {
                let include_path: PathBuf = p.0.as_ref().into();
                let target = path
                    .parent()
                    .ok_or_else(|| LoadError::IncludePath(path.to_owned()))?
                    .join(include_path);
                load_repl_impl(&target, callback)
            }
            other => callback(path, &other),
        }?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parse::{self, parse_ledger};

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::{path::Path, vec::Vec};

    fn parse_static_repl(
        input: &'static str,
    ) -> Result<Vec<parse::ParsedLedgerEntry<'static>>, parse::ParseError> {
        parse_ledger(input).collect()
    }

    #[test]
    fn load_valid_input() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/root.ledger");
        let child1 = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/child1.ledger");
        let child2 = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/child2.ledger");
        let mut i: usize = 0;
        let want = parse_static_repl(indoc! {"
            account Expenses:Grocery
                note スーパーマーケットで買ったやつ全部
                ; comment
                alias Expenses:CVS

            2024/1/1 Initial Balance
                Equity:Opening Balance       -1000.00 CHF
                Assets:Bank:ZKB               1000.00 CHF

            2024/1/1 * SBB CFF FFS
                Assets:Bank:ZKB                 -5.60 CHF
                Expenses:Travel:Train            5.60 CHF

            2024/5/1 * Migros
                Expenses:Grocery               -10.00 CHF
                Assets:Bank:ZKB                 10.00 CHF
        "})
        .expect("test input parse must not fail");
        load_repl(&root, |path, entry| {
            if i < 2 {
                assert_eq!(path, &root);
            } else if i < 3 {
                assert_eq!(path, &child2);
            } else if i < 4 {
                assert_eq!(path, &child1);
            } else {
                panic!("unxpected got index {}", i);
            }
            let want = want.get(i).expect("missing want anymore, too many got");
            assert_eq!(want, entry);
            i += 1;
            Ok::<(), LoadError>(())
        })
        .expect("test_failed");
    }
}
