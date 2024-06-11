//! Module `load` contains the functions useful for loading Ledger file,
//! recursively resolving the `include` directives.

use crate::repl;

use std::path::PathBuf;

/// Error caused by `load_*` functions.
#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse file {0}")]
    Parse(#[from] repl::parser::ParseLedgerError),
    #[error("unexpected include path {0}, maybe filesystem root is passed")]
    IncludePath(PathBuf),
}

/// Returns `repl` format of ledger file entry, recursively resolving `include` directives.
pub fn load_repl(path: &std::path::Path) -> Result<Vec<repl::LedgerEntry>, LoadError> {
    let mut r = Vec::new();
    load_repl_impl(path, &mut r)?;
    Ok(r)
}

fn load_repl_impl(
    path: &std::path::Path,
    ret: &mut Vec<repl::LedgerEntry>,
) -> Result<(), LoadError> {
    let content = std::fs::read_to_string(path)?;
    let vs = repl::parser::parse_ledger(&content)?;
    for elem in vs.into_iter() {
        match elem {
            repl::LedgerEntry::Include(p) => {
                let include_path: PathBuf = p.0.into();
                let target = path
                    .parent()
                    .ok_or_else(|| LoadError::IncludePath(path.to_owned()))?
                    .join(include_path);
                load_repl_impl(target.as_path(), ret)?
            }
            _ => ret.push(elem),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::repl::parser::parse_ledger;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn load_valid_input() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/root.ledger");
        let got = load_repl(&root).unwrap();
        let want = parse_ledger(indoc! {"
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
        .unwrap();
        assert_eq!(got, want);
    }
}
