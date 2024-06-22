//! Module `load` contains the functions useful for loading Ledger file,
//! recursively resolving the `include` directives.

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{parse, repl};

/// Error caused by `load_*` functions.
#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse file")]
    Parse(#[from] parse::ParseError),
    #[error("unexpected include path {0}, maybe filesystem root is passed")]
    IncludePath(PathBuf),
}

/// Loader is an object to keep loading a given file and may recusrively load them as `repr::LedgerEntry`,
/// with the metadata about filename or line/column to point the error in a user friendly manner.
pub struct Loader {
    source: PathBuf,
    error_style: annotate_snippets::Renderer,
    filesystem: FileSystem,
}

impl Loader {
    /// Create a new instance of `Loader` to load the given path.
    ///
    /// It might look weird to have the source path as a `Loader` member,
    /// but that would give future flexibility to support loading from stdio/network without include,
    /// or completely static one.
    pub fn new(source: PathBuf) -> Self {
        Self {
            source,
            // TODO: use plain by default.
            error_style: annotate_snippets::Renderer::styled(),
            filesystem: FileSystem::Prod,
        }
    }

    /// Set `fake_files` which resolves Path -> its contents conversion.
    pub fn with_fake_files(&mut self, fake_files: HashMap<PathBuf, Vec<u8>>) -> &mut Self {
        self.filesystem = FileSystem::Fake(fake_files);
        self
    }

    /// Loads `repl::LedgerEntry` and invoke callback on every entry,
    /// recursively resolving `include` directives.
    pub fn load_repl<T, E>(&self, mut callback: T) -> Result<(), E>
    where
        T: FnMut(&Path, &parse::ParsedLedgerEntry<'_>) -> Result<(), E>,
        E: std::error::Error + From<LoadError>,
    {
        let popts = parse::ParseOptions::default().with_error_style(self.error_style.clone());
        self.load_repl_impl(&popts, &self.source, &mut callback)
    }

    fn load_repl_impl<T, E>(
        &self,
        parse_options: &parse::ParseOptions,
        path: &Path,
        callback: &mut T,
    ) -> Result<(), E>
    where
        T: FnMut(&Path, &parse::ParsedLedgerEntry<'_>) -> Result<(), E>,
        E: std::error::Error + From<LoadError>,
    {
        let path = self.filesystem.canonicalize_path(path);
        let content = self
            .filesystem
            .file_content_utf8(&path)
            .map_err(LoadError::IO)?;
        for entry in parse_options.parse_ledger(&content) {
            match entry.map_err(LoadError::Parse)? {
                repl::LedgerEntry::Include(p) => {
                    let include_path: PathBuf = p.0.as_ref().into();
                    let target = path
                        .as_ref()
                        .parent()
                        .ok_or_else(|| LoadError::IncludePath(path.as_ref().to_owned()))?
                        .join(include_path);
                    self.load_repl_impl(&parse_options, &target, callback)
                }
                other => callback(&path, &other),
            }?;
        }
        Ok(())
    }
}

enum FileSystem {
    Prod,
    Fake(HashMap<PathBuf, Vec<u8>>),
}

impl FileSystem {
    fn canonicalize_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        std::fs::canonicalize(path)
            .map(Cow::Owned)
            .unwrap_or_else(|x| {
                log::warn!(
                    "failed to canonicalize path {}, likeky to fail to load: {}",
                    path.display(),
                    x
                );
                path.into()
            })
    }

    fn file_content_utf8<P: AsRef<Path>>(&self, path: P) -> Result<String, std::io::Error> {
        let path = path.as_ref();
        match self {
            FileSystem::Prod => std::fs::read_to_string(path),
            FileSystem::Fake(fake_fs) => fake_fs
                .get(path)
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("fake file {} not found", path.display()),
                ))
                .and_then(|x| {
                    String::from_utf8(x.clone())
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parse::{self, ParseOptions};

    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use std::{path::Path, vec::Vec};

    fn parse_static_repl<'a>(
        input: &[(&'a Path, &'static str)],
    ) -> Result<Vec<(&'a Path, parse::ParsedLedgerEntry<'static>)>, parse::ParseError> {
        let opts = ParseOptions::default();
        input
            .iter()
            .flat_map(|(p, content)| opts.parse_ledger(content).map(|elem| elem.map(|x| (*p, x))))
            .collect()
    }

    #[test]
    fn load_valid_input_real_file() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/root.ledger");
        let child1 = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/child1.ledger");
        let child2 = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/sub/child2.ledger");
        let child3 = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/child3.ledger");
        let mut i: usize = 0;
        let want = parse_static_repl(&[
            (
                &root,
                indoc! {"
            account Expenses:Grocery
                note スーパーマーケットで買ったやつ全部
                ; comment
                alias Expenses:CVS

            2024/01/01 Initial Balance
                Equity:Opening Balance                  -1000.00 CHF
                Assets:Bank:ZKB                          1000.00 CHF
            "},
            ),
            (
                &child2,
                indoc! {"
            2024/01/01 * Complicated salary
                Income:Salary                          -3,000.00 CHF
                Assets:Bank:ZKB                         2,500.00 CHF
                Expenses:Income Tax                       312.34 CHF
                Expenses:Social Tax                        37.66 CHF
                Assets:Fixed:年金                         150.00 CHF
            "},
            ),
            (
                &child3,
                indoc! {"
            2024/03/01 * SBB CFF FFS
                Assets:Bank:ZKB                            -5.60 CHF
                Expenses:Travel:Train                       5.60 CHF
            "},
            ),
            (
                &child2,
                indoc! {"
            2024/01/25 ! RSU
                ; TODO: FMV not determined
                Income:RSU                    (-50.0000 * 100.23 USD)
                Expenses:Income Tax
                Assets:Broker                            40.0000 OKANE @ 100.23 USD
            "},
            ),
            (
                &child1,
                indoc! {"
            2024/05/01 * Migros
                Expenses:Grocery                          -10.00 CHF
                Assets:Bank:ZKB                            10.00 CHF
            "},
            ),
        ])
        .expect("test input parse must not fail");
        Loader::new(root.clone())
            .load_repl(|path, entry| {
                let (want_path, want_entry) =
                    want.get(i).expect("missing want anymore, too many got");
                assert_eq!((*want_path, want_entry), (path, entry));
                i += 1;
                Ok::<(), LoadError>(())
            })
            .expect("test_failed");
    }

    #[test]
    fn load_valid_fake() {
        let fake = hashmap! {
            PathBuf::from("/path/to/root.ledger") => indoc! {"
                include child1.ledger
            "}.as_bytes().to_vec(),
            PathBuf::from("/path/to/child1.ledger") => indoc! {"
                include sub/child2.ledger
            "}.as_bytes().to_vec(),
            PathBuf::from("/path/to/sub/child2.ledger") => indoc! {"
                include child3.ledger
            "}.as_bytes().to_vec(),
            PathBuf::from("/path/to/sub/child3.ledger") => indoc! {"
                ; comment here
            "}.as_bytes().to_vec(),
        };
        let mut i: usize = 0;
        let want = parse_static_repl(&[(
            Path::new("/path/to/sub/child3.ledger"),
            indoc! {"
            ; comment here
            "},
        )])
        .expect("test input parse must not fail");
        Loader::new(PathBuf::from("/path/to/root.ledger"))
            .with_fake_files(fake)
            .load_repl(|path, entry| {
                let (want_path, want_entry) =
                    want.get(i).expect("missing want anymore, too many got");
                assert_eq!((*want_path, want_entry), (path, entry));
                i += 1;
                Ok::<(), LoadError>(())
            })
            .expect("test_failed");
    }
}
