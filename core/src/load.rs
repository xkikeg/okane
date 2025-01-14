//! Contains the functions to load Ledger file,
//! with recursively resolving the `include` directives.

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{self, Path, PathBuf},
};

use crate::{parse, syntax};

/// Error caused by [Loader::load].
#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse file {1}")]
    Parse(#[source] Box<parse::ParseError>, PathBuf),
    #[error("loading file path {0} doesn't have parent, maybe filesystem root is passed")]
    RootLoadingPath(PathBuf),
    #[error("invalid Unicode path is not supported: {0}")]
    InvalidUnicodePath(String),
    #[error("invalid glob pattern specified")]
    InvalidIncludeGlob(#[from] glob::PatternError),
    #[error("failed to match glob pattern")]
    GlobFailure(#[from] glob::GlobError),
}

/// Loader is an object to keep loading a given file and may recusrively load them as `repr::LedgerEntry`,
/// with the metadata about filename or line/column to point the error in a user friendly manner.
pub struct Loader<F: FileSystem> {
    source: PathBuf,
    error_style: annotate_snippets::Renderer,
    filesystem: F,
}

/// Creates a new [`Loader`] instance with [`ProdFileSystem`].
pub fn new_loader(source: PathBuf) -> Loader<ProdFileSystem> {
    Loader::new(source, ProdFileSystem)
}

impl<F: FileSystem> Loader<F> {
    /// Create a new instance of `Loader` to load the given path.
    ///
    /// It might look weird to have the source path as a `Loader` member,
    /// but that would give future flexibility to support loading from stdio/network without include,
    /// or completely static one.
    pub fn new(source: PathBuf, filesystem: F) -> Self {
        Self {
            source,
            // TODO: use plain by default.
            error_style: annotate_snippets::Renderer::styled(),
            filesystem,
        }
    }

    /// Returns a [`annotate_snippets::Renderer`] instance.
    pub(crate) fn error_style(&self) -> &annotate_snippets::Renderer {
        &self.error_style
    }

    /// Loads [syntax::LedgerEntry] and invoke callback on every entry,
    /// recursively resolving `include` directives.
    pub fn load<T, E, Deco>(&self, mut callback: T) -> Result<(), E>
    where
        T: FnMut(&Path, &parse::ParsedContext<'_>, &syntax::LedgerEntry<'_, Deco>) -> Result<(), E>,
        E: std::error::Error + From<LoadError>,
        Deco: syntax::decoration::Decoration,
    {
        let popts = parse::ParseOptions::default().with_error_style(self.error_style.clone());
        self.load_impl(&popts, &self.source, &mut callback)
    }

    fn load_impl<T, E, Deco>(
        &self,
        parse_options: &parse::ParseOptions,
        path: &Path,
        callback: &mut T,
    ) -> Result<(), E>
    where
        T: FnMut(&Path, &parse::ParsedContext<'_>, &syntax::LedgerEntry<'_, Deco>) -> Result<(), E>,
        E: std::error::Error + From<LoadError>,
        Deco: syntax::decoration::Decoration,
    {
        let path: Cow<'_, Path> = self.filesystem.canonicalize_path(path);
        let content = self
            .filesystem
            .file_content_utf8(&path)
            .map_err(LoadError::IO)?;
        for parsed in parse_options.parse_ledger(&content) {
            let (ctx, entry) =
                parsed.map_err(|e| LoadError::Parse(e, path.clone().into_owned()))?;
            match entry {
                syntax::LedgerEntry::Include(p) => {
                    let include_path: PathBuf = p.0.as_ref().into();
                    let target: String = path
                        .as_ref()
                        .parent()
                        .ok_or_else(|| LoadError::RootLoadingPath(path.as_ref().to_owned()))?
                        .join(include_path)
                        .into_os_string()
                        .into_string()
                        .map_err(|x| {
                            LoadError::InvalidUnicodePath(format!("{}", PathBuf::from(x).display()))
                        })?;
                    let mut paths: Vec<PathBuf> = self.filesystem.glob(&target)?;
                    paths.sort_unstable();
                    for path in &paths {
                        self.load_impl(parse_options, path, callback)?;
                    }
                    Ok(())
                }
                _ => callback(&path, &ctx, &entry),
            }?;
        }
        Ok(())
    }
}

/// Interface to abstract file system.
/// Normally you want to use [ProdFileSystem].
pub trait FileSystem {
    /// canonicalize the given path.
    fn canonicalize_path<'a>(&self, path: &'a Path) -> Cow<'a, Path>;

    /// Load the given path and returns it as UTF-8 String.
    fn file_content_utf8<P: AsRef<Path>>(&self, path: P) -> Result<String, std::io::Error>;

    /// Returns all paths matching the given glob.
    /// Paths can be in arbitrary order, and caller must sort it beforehand.
    fn glob(&self, pattern: &str) -> Result<Vec<PathBuf>, LoadError>;
}

/// [FileSystem] to regularly reads the files recursively in the local files.
pub struct ProdFileSystem;

impl FileSystem for ProdFileSystem {
    fn canonicalize_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        std::fs::canonicalize(path)
            .map(|x| {
                if x == path {
                    Cow::Borrowed(path)
                } else {
                    Cow::Owned(x)
                }
            })
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
        std::fs::read_to_string(path)
    }

    fn glob(&self, pattern: &str) -> Result<Vec<PathBuf>, LoadError> {
        let paths: Vec<PathBuf> =
            glob::glob(pattern)?.collect::<Result<Vec<_>, glob::GlobError>>()?;
        Ok(paths)
    }
}

/// [FileSystem] with given set of filename and content mapping.
/// It won't cause any actual file read.
pub struct FakeFileSystem(HashMap<PathBuf, Vec<u8>>);

impl From<HashMap<PathBuf, Vec<u8>>> for FakeFileSystem {
    fn from(value: HashMap<PathBuf, Vec<u8>>) -> Self {
        Self(value)
    }
}

impl FileSystem for FakeFileSystem {
    fn canonicalize_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        let mut ret = PathBuf::new();
        for pc in path.components() {
            match pc {
                path::Component::CurDir => (),
                path::Component::ParentDir => {
                    if !ret.pop() {
                        log::warn!(
                            "failed to pop parent, maybe wrong path given: {}",
                            path.display()
                        );
                    }
                }
                _ => ret.push(pc),
            }
        }
        Cow::Owned(ret)
    }

    fn file_content_utf8<P: AsRef<Path>>(&self, path: P) -> Result<String, std::io::Error> {
        let path = path.as_ref();
        self.0
            .get(path)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("fake file {} not found", path.display()),
            ))
            .and_then(|x| {
                String::from_utf8(x.clone())
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            })
    }

    fn glob(&self, pattern: &str) -> Result<Vec<PathBuf>, LoadError> {
        let pattern = glob::Pattern::new(pattern)?;
        let mut paths: Vec<PathBuf> = self
            .0
            .keys()
            .filter(|x| pattern.matches_path(x))
            .cloned()
            .collect();
        paths.sort_by(|x, y| y.cmp(x));
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Borrow, path::Path, vec::Vec};

    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::parse::{self, ParseOptions};

    use super::*;

    fn parse_static_ledger_entry(
        input: &[(&Path, &'static str)],
    ) -> Result<Vec<(PathBuf, syntax::plain::LedgerEntry<'static>)>, Box<parse::ParseError>> {
        let opts = ParseOptions::default();
        input
            .iter()
            .flat_map(|(p, content)| {
                opts.parse_ledger(content)
                    .map(|elem| elem.map(|(_ctx, entry)| (p.to_path_buf(), entry)))
            })
            .collect()
    }

    fn parse_into_vec<L, F>(
        loader: L,
    ) -> Result<Vec<(PathBuf, syntax::plain::LedgerEntry<'static>)>, LoadError>
    where
        L: Borrow<Loader<F>>,
        F: FileSystem,
    {
        let mut ret: Vec<(PathBuf, syntax::plain::LedgerEntry<'static>)> = Vec::new();
        loader.borrow().load(|path, _ctx, entry| {
            ret.push((path.to_owned(), entry.to_static()));
            Ok::<(), LoadError>(())
        })?;
        Ok(ret)
    }

    #[test]
    fn load_valid_input_real_file() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/root.ledger")
            .canonicalize()
            .unwrap();
        let child1 = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/child1.ledger")
            .canonicalize()
            .unwrap();
        let child2 = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/sub/child2.ledger")
            .canonicalize()
            .unwrap();
        let child3 = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/child3.ledger")
            .canonicalize()
            .unwrap();
        let child4 = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/sub/child4.ledger")
            .canonicalize()
            .unwrap();
        let want = parse_static_ledger_entry(&[
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
                &child4,
                indoc! {"
            2024/7/1 * Send money
                Assets:Bank:ZKB                         -1000.00 CHF
                Assets:Wire:Wise                         1000.00 CHF
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
        let got = parse_into_vec(new_loader(root.clone())).expect("failed to parse the test data");
        assert_eq!(want, got);
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
        let want = parse_static_ledger_entry(&[(
            Path::new("/path/to/sub/child3.ledger"),
            indoc! {"
            ; comment here
            "},
        )])
        .expect("test input parse must not fail");
        let got = parse_into_vec(Loader::new(
            PathBuf::from("/path/to/root.ledger"),
            FakeFileSystem::from(fake),
        ))
        .expect("parse failed");
        assert_eq!(want, got);
    }
}
