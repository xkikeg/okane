//! Config files that give defaults for command line flags.
//!
//! Each key is the long name of the flag it stands for, without `--`, and
//! each table is the subcommand the flags belong to. `[report]` is shared by
//! `balance`, `register` and `ui`, and each of those can override it in its
//! own table. Flags given on the command line always win.
//!
//! Files are read, from weakest to strongest:
//!
//! 1. the user config, `$XDG_CONFIG_HOME/okane/config.toml`
//!    (`~/.config/okane/config.toml` when unset, `%APPDATA%\okane\config.toml` on Windows);
//! 2. the project config, the nearest `okane.toml` in the current directory or
//!    its parents, stopping at the home directory.
//!
//! `OKANE_CONFIG=path` reads only that file instead, and `--no-config` reads none.

use std::io;
use std::path::{Component, Path, PathBuf};

use anyhow::Context as _;
use serde::Deserialize;

use crate::cmd::{AccountFilterMode, ConversionMode, SortKey};
use crate::merge::{Merge, MergeOwned, impl_merge_from_owned};

/// File name of the project config.
const PROJECT_FILE_NAME: &str = "okane.toml";

/// Error while finding or reading the config files.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file {}", .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("OKANE_CONFIG points to {}, which does not exist", .0.display())]
    ExplicitNotFound(PathBuf),
    #[error("invalid config file {}: {source}", .path.display())]
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
}

/// Merged contents of the config files.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    /// Default Ledger file, used by commands reading one.
    pub source: Option<PathBuf>,
    /// Defaults shared by `balance`, `register` and `ui`.
    report: ReportTable,
    balance: ReportTable,
    register: RegisterTable,
    ui: ReportTable,
    pub import: ImportTable,
}

/// Flags shared by the report commands.
/// This is used in `[report]`, as well as actual report commands
/// such as`[balance]` or `[register]` .
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ReportTable {
    pub price_db: Option<PathBuf>,
    pub exchange: Option<String>,
    pub conversion: Option<ConversionMode>,
    pub account_filter: Option<AccountFilterMode>,
}

/// Config table for `[register]`.
///
/// [`ReportTable`] plus the register only flags.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct RegisterTable {
    #[serde(flatten)]
    pub report: ReportTable,
    pub sort: Option<SortKey>,
}

/// `[import]`.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ImportTable {
    pub rules: Option<PathBuf>,
    /// Ledger for `--interactive` must be stored
    pub ledger: Option<PathBuf>,
}

impl MergeOwned for ReportTable {
    fn merge_owned(self, other: Self) -> Self {
        Self {
            price_db: other.price_db.or(self.price_db),
            exchange: other.exchange.or(self.exchange),
            conversion: other.conversion.or(self.conversion),
            account_filter: other.account_filter.or(self.account_filter),
        }
    }
}

impl_merge_from_owned!(ReportTable);

impl MergeOwned for RegisterTable {
    fn merge_owned(self, other: Self) -> Self {
        Self {
            report: self.report.merge(other.report),
            sort: other.sort.or(self.sort),
        }
    }
}

impl_merge_from_owned!(RegisterTable);

impl MergeOwned for ImportTable {
    fn merge_owned(self, other: Self) -> Self {
        Self {
            rules: other.rules.or(self.rules),
            ledger: other.ledger.or(self.ledger),
        }
    }
}

impl_merge_from_owned!(ImportTable);

impl MergeOwned for Config {
    fn merge_owned(self, other: Self) -> Self {
        Self {
            source: other.source.or(self.source),
            report: self.report.merge(other.report),
            balance: self.balance.merge(other.balance),
            register: self.register.merge(other.register),
            ui: self.ui.merge(other.ui),
            import: self.import.merge(other.import),
        }
    }
}

impl_merge_from_owned!(Config);

#[cfg(test)]
#[derive(Default)]
pub(super) struct ConfigBuilder {
    config: Config,
}

#[cfg(test)]
impl ConfigBuilder {
    pub fn source(mut self, source: impl Into<PathBuf>) -> Self {
        self.config.source = Some(source.into());
        self
    }

    pub fn report(mut self, report: ReportTable) -> Self {
        self.config.report = report;
        self
    }

    pub fn balance(mut self, balance: ReportTable) -> Self {
        self.config.balance = balance;
        self
    }

    pub fn register(mut self, register: RegisterTable) -> Self {
        self.config.register = register;
        self
    }

    pub fn ui(mut self, ui: ReportTable) -> Self {
        self.config.ui = ui;
        self
    }

    pub fn import(mut self, import: ImportTable) -> Self {
        self.config.import = import;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

impl Config {
    #[cfg(test)]
    pub(super) fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Effective `[balance]`, falling back to `[report]`.
    pub fn balance(&self) -> ReportTable {
        self.report.clone().merge(self.balance.clone())
    }

    /// Effective report keys of `[register]`, falling back to `[report]`.
    pub fn register(&self) -> RegisterTable {
        RegisterTable {
            report: self.report.clone().merge(self.register.report.clone()),
            sort: self.register.sort,
        }
    }

    /// Effective report keys of `[ui]`, falling back to `[report]`.
    pub fn ui(&self) -> ReportTable {
        self.report.clone().merge(self.ui.clone())
    }

    /// Ledger for `import --interactive`.
    pub fn import_ledger(&self) -> Option<&Path> {
        self.import.ledger.as_deref().or(self.source.as_deref())
    }
}

/// Makes the paths in a config file absolute: relative ones are relative to
/// `base`, the directory of the file, and `~` stands for `home`.
struct PathResolver<'a> {
    base: &'a Path,
    home: Option<&'a Path>,
}

impl PathResolver<'_> {
    fn resolve(&self, path: &mut Option<PathBuf>) {
        if let Some(path) = path {
            *path = self.resolved(path);
        }
    }

    fn resolved(&self, path: &Path) -> PathBuf {
        let mut components = path.components();
        match (components.next(), self.home) {
            (Some(Component::Normal(first)), Some(home)) if first == "~" => {
                home.join(components.as_path())
            }
            _ => self.base.join(path),
        }
    }
}

/// Config tables holding paths, which [`PathResolver`] makes absolute.
///
/// Implementations destructure `self` without `..`, so that a new field
/// does not compile until it is either resolved or ignored on purpose.
trait ResolvePaths {
    fn resolve_paths(&mut self, resolver: &PathResolver);
}

impl ResolvePaths for Config {
    fn resolve_paths(&mut self, resolver: &PathResolver) {
        let Config {
            source,
            report,
            balance,
            register,
            ui,
            import,
        } = self;
        resolver.resolve(source);
        report.resolve_paths(resolver);
        balance.resolve_paths(resolver);
        register.resolve_paths(resolver);
        ui.resolve_paths(resolver);
        import.resolve_paths(resolver);
    }
}

impl ResolvePaths for ReportTable {
    fn resolve_paths(&mut self, resolver: &PathResolver) {
        let ReportTable {
            price_db,
            exchange: _,
            conversion: _,
            account_filter: _,
        } = self;
        resolver.resolve(price_db);
    }
}

impl ResolvePaths for RegisterTable {
    fn resolve_paths(&mut self, resolver: &PathResolver) {
        let RegisterTable { report, sort: _ } = self;
        report.resolve_paths(resolver);
    }
}

impl ResolvePaths for ImportTable {
    fn resolve_paths(&mut self, resolver: &PathResolver) {
        let ImportTable { rules, ledger } = self;
        resolver.resolve(rules);
        resolver.resolve(ledger);
    }
}

/// Env var `OKANE_CONFIG` state.
#[derive(Debug, Default, Clone)]
pub enum OkaneConfigEnvVar {
    /// The var isn't set. We don't look it up.
    #[default]
    Unset,
    /// Empty, the same effect as `--no-config`.
    Empty,
    /// Actual path.
    Set(PathBuf),
}

impl OkaneConfigEnvVar {
    fn from_os_var() -> Self {
        match std::env::var_os("OKANE_CONFIG") {
            Some(v) if v.is_empty() => Self::Empty,
            Some(v) => Self::Set(PathBuf::from(v)),
            None => Self::Unset,
        }
    }
}

/// Environment the config files are found in.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// Current directory, where the search for `okane.toml` starts.
    pub cwd: PathBuf,
    pub home: Option<PathBuf>,
    /// Directory holding the user config directory `okane`.
    pub config_home: Option<PathBuf>,
    /// `$OKANE_CONFIG`.
    pub okane_config: OkaneConfigEnvVar,
}

impl Env {
    /// Reads the environment of this process.
    pub fn from_process() -> anyhow::Result<Self> {
        let var = |name: &str| {
            std::env::var_os(name)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        let home = std::env::home_dir().filter(|p| !p.as_os_str().is_empty());
        Ok(Env {
            cwd: std::env::current_dir().context("failed to read the current directory")?,
            config_home: config_home(cfg!(windows), var, home.as_deref()),
            home,
            okane_config: OkaneConfigEnvVar::from_os_var(),
        })
    }

    /// Path of the user config.
    fn user_config(&self) -> Option<PathBuf> {
        Some(self.config_home.as_ref()?.join("okane").join("config.toml"))
    }
}

/// `%APPDATA%` on Windows, otherwise `$XDG_CONFIG_HOME`, falling back to
/// `~/.config` as the XDG spec says.
fn config_home(
    windows: bool,
    var: impl Fn(&str) -> Option<PathBuf>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if windows {
        return var("APPDATA");
    }
    // The XDG spec says to ignore relative paths.
    var("XDG_CONFIG_HOME")
        .filter(|p| p.is_absolute())
        .or_else(|| Some(home?.join(".config")))
}

/// Reads the files config is read from.
pub trait ConfigFs {
    /// Returns the content of `path`, or `None` if it does not exist.
    fn read(&self, path: &Path) -> io::Result<Option<String>>;
}

/// [`ConfigFs`] on the real file system.
pub struct RealFs;

impl ConfigFs for RealFs {
    fn read(&self, path: &Path) -> io::Result<Option<String>> {
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Finds, reads and merges the config files.
pub fn load(env: &Env, fs: &impl ConfigFs) -> Result<Config, ConfigError> {
    match &env.okane_config {
        OkaneConfigEnvVar::Unset => (),
        OkaneConfigEnvVar::Empty => return Ok(Config::default()),
        OkaneConfigEnvVar::Set(path) => {
            let path = env.cwd.join(path);
            let content =
                read(fs, &path)?.ok_or_else(|| ConfigError::ExplicitNotFound(path.clone()))?;
            return parse(&path, &content, env);
        }
    }
    let mut config = Config::default();
    if let Some(path) = env.user_config()
        && let Some(content) = read(fs, &path)?
    {
        config.merge_from(parse(&path, &content, env)?);
    }
    if let Some((path, content)) = find_project(env, fs)? {
        config.merge_from(parse(&path, &content, env)?);
    }
    Ok(config)
}

fn read(fs: &impl ConfigFs, path: &Path) -> Result<Option<String>, ConfigError> {
    fs.read(path).map_err(|source| ConfigError::Read {
        path: path.to_owned(),
        source,
    })
}

fn parse(path: &Path, content: &str, env: &Env) -> Result<Config, ConfigError> {
    let mut config: Config = toml::from_str(content).map_err(|source| ConfigError::Parse {
        path: path.to_owned(),
        source: Box::new(source),
    })?;
    log::info!("loaded config file {}", path.display());
    config.resolve_paths(&PathResolver {
        base: path.parent().unwrap_or(Path::new("")),
        home: env.home.as_deref(),
    });
    Ok(config)
}

/// Finds the nearest `okane.toml`, from the current directory up to the home
/// directory, or to the root if the current directory is not under home.
fn find_project(env: &Env, fs: &impl ConfigFs) -> Result<Option<(PathBuf, String)>, ConfigError> {
    for dir in env.cwd.ancestors() {
        let path = dir.join(PROJECT_FILE_NAME);
        if let Some(content) = read(fs, &path)? {
            return Ok(Some((path, content)));
        }
        if env.home.as_deref() == Some(dir) {
            break;
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[derive(Default)]
    struct FakeFs(HashMap<PathBuf, String>);

    impl FakeFs {
        fn with(mut self, path: &str, content: &str) -> Self {
            self.0.insert(PathBuf::from(path), content.to_owned());
            self
        }
    }

    impl ConfigFs for FakeFs {
        fn read(&self, path: &Path) -> io::Result<Option<String>> {
            Ok(self.0.get(path).cloned())
        }
    }

    fn env(cwd: &str) -> Env {
        Env {
            cwd: PathBuf::from(cwd),
            home: Some(PathBuf::from("/home/u")),
            config_home: Some(PathBuf::from("/home/u/.config")),
            okane_config: OkaneConfigEnvVar::Unset,
        }
    }

    const USER: &str = "/home/u/.config/okane/config.toml";

    /// [`USER`] as error messages print it: joined by the platform's
    /// separator, so `/home/u/.config\okane\config.toml` on Windows.
    fn user_display() -> String {
        Path::new("/home/u/.config")
            .join("okane")
            .join("config.toml")
            .display()
            .to_string()
    }

    #[test]
    fn no_files_give_the_default() {
        let config = load(&env("/home/u/books"), &FakeFs::default()).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn parses_every_key() {
        let fs = FakeFs::default().with(
            USER,
            indoc! {r#"
                source = "main.ledger"

                [report]
                price-db = "prices.db"
                exchange = "JPY"
                conversion = "historical"
                account-filter = "exact"

                [balance]
                exchange = "CHF"

                [register]
                price-db = "reg.db"
                sort = "date"

                [ui]
                conversion = "up-to-date"

                [import]
                rules = "import.yml"
                ledger = "accounts.ledger"
            "#},
        );
        let config = load(&env("/tmp"), &fs).unwrap();
        let dir = Path::new("/home/u/.config/okane");
        assert_eq!(
            config,
            Config {
                source: Some(dir.join("main.ledger")),
                report: ReportTable {
                    price_db: Some(dir.join("prices.db")),
                    exchange: Some("JPY".to_owned()),
                    conversion: Some(ConversionMode::Historical),
                    account_filter: Some(AccountFilterMode::Exact),
                },
                balance: ReportTable {
                    exchange: Some("CHF".to_owned()),
                    ..ReportTable::default()
                },
                register: RegisterTable {
                    report: ReportTable {
                        price_db: Some(dir.join("reg.db")),
                        ..ReportTable::default()
                    },
                    sort: Some(SortKey::Date),
                },
                ui: ReportTable {
                    conversion: Some(ConversionMode::UpToDate),
                    ..ReportTable::default()
                },
                import: ImportTable {
                    rules: Some(dir.join("import.yml")),
                    ledger: Some(dir.join("accounts.ledger")),
                },
            }
        );
    }

    #[test]
    fn merge_config() {
        let base = Config::builder()
            .source("/base.ledger")
            .report(ReportTable {
                price_db: Some(PathBuf::from("/base.db")),
                exchange: Some("JPY".to_owned()),
                conversion: Some(ConversionMode::Historical),
                account_filter: Some(AccountFilterMode::Regex),
            })
            .balance(ReportTable {
                exchange: Some("CHF".to_owned()),
                ..ReportTable::default()
            })
            .register(RegisterTable {
                report: ReportTable {
                    exchange: Some("EUR".to_owned()),
                    ..ReportTable::default()
                },
                sort: Some(SortKey::Original),
            })
            .ui(ReportTable {
                conversion: Some(ConversionMode::UpToDate),
                ..ReportTable::default()
            })
            .import(ImportTable {
                rules: Some(PathBuf::from("/base.yml")),
                ledger: Some(PathBuf::from("/base-import.ledger")),
            })
            .build();
        let over = Config::builder()
            .report(ReportTable {
                exchange: Some("USD".to_owned()),
                ..ReportTable::default()
            })
            .register(RegisterTable {
                report: ReportTable {
                    conversion: Some(ConversionMode::UpToDate),
                    ..ReportTable::default()
                },
                sort: Some(SortKey::Date),
            })
            .ui(ReportTable {
                exchange: Some("GBP".to_owned()),
                ..ReportTable::default()
            })
            .import(ImportTable {
                rules: Some(PathBuf::from("/over.yml")),
                ledger: None,
            })
            .build();
        assert_eq!(base, base.clone().merge(Config::default()));
        assert_eq!(base, Config::default().merge(base.clone()));
        assert_eq!(
            Config {
                source: Some(PathBuf::from("/base.ledger")),
                report: ReportTable {
                    price_db: Some(PathBuf::from("/base.db")),
                    exchange: Some("USD".to_owned()),
                    conversion: Some(ConversionMode::Historical),
                    account_filter: Some(AccountFilterMode::Regex),
                },
                balance: ReportTable {
                    exchange: Some("CHF".to_owned()),
                    ..ReportTable::default()
                },
                register: RegisterTable {
                    report: ReportTable {
                        exchange: Some("EUR".to_owned()),
                        conversion: Some(ConversionMode::UpToDate),
                        ..ReportTable::default()
                    },
                    sort: Some(SortKey::Date),
                },
                ui: ReportTable {
                    exchange: Some("GBP".to_owned()),
                    conversion: Some(ConversionMode::UpToDate),
                    ..ReportTable::default()
                },
                import: ImportTable {
                    rules: Some(PathBuf::from("/over.yml")),
                    ledger: Some(PathBuf::from("/base-import.ledger")),
                },
            },
            base.merge(over)
        );
    }

    #[test]
    fn command_tables_fall_back_to_report() {
        let fs = FakeFs::default().with(
            USER,
            indoc! {r#"
                [report]
                exchange = "JPY"
                conversion = "historical"
                [balance]
                exchange = "CHF"
                [register]
                conversion = "up-to-date"
            "#},
        );
        let config = load(&env("/tmp"), &fs).unwrap();
        assert_eq!(config.balance().exchange.as_deref(), Some("CHF"));
        assert_eq!(
            config.balance().conversion,
            Some(ConversionMode::Historical)
        );
        assert_eq!(config.register().report.exchange.as_deref(), Some("JPY"));
        assert_eq!(
            config.register().report.conversion,
            Some(ConversionMode::UpToDate)
        );
        assert_eq!(config.ui().exchange.as_deref(), Some("JPY"));
    }

    #[test]
    fn project_overrides_user_per_key() {
        let fs = FakeFs::default()
            .with(
                USER,
                indoc! {r#"
                    source = "~/main.ledger"
                    [report]
                    exchange = "JPY"
                    price-db = "~/prices.db"
                "#},
            )
            .with(
                "/home/u/books/okane.toml",
                indoc! {r#"
                    [report]
                    price-db = "prices.db"
                "#},
            );
        let config = load(&env("/home/u/books/2026"), &fs).unwrap();
        assert_eq!(config.source, Some(PathBuf::from("/home/u/main.ledger")));
        assert_eq!(config.report.exchange.as_deref(), Some("JPY"));
        assert_eq!(
            config.report.price_db,
            Some(PathBuf::from("/home/u/books/prices.db"))
        );
    }

    #[test]
    fn project_search_stops_at_home() {
        let fs = FakeFs::default().with("/home/okane.toml", "source = \"x\"");
        let config = load(&env("/home/u/books"), &fs).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn project_search_checks_home_itself() {
        let fs = FakeFs::default().with("/home/u/okane.toml", "source = \"x\"");
        let config = load(&env("/home/u/books"), &fs).unwrap();
        assert_eq!(config.source, Some(PathBuf::from("/home/u/x")));
    }

    #[test]
    fn project_search_outside_home_goes_up_to_root() {
        let fs = FakeFs::default().with("/okane.toml", "source = \"x\"");
        let config = load(&env("/srv/books"), &fs).unwrap();
        assert_eq!(config.source, Some(PathBuf::from("/x")));
    }

    #[test]
    fn nearest_project_wins() {
        let fs = FakeFs::default()
            .with("/home/u/okane.toml", "source = \"outer\"")
            .with("/home/u/books/okane.toml", "source = \"inner\"");
        let config = load(&env("/home/u/books"), &fs).unwrap();
        assert_eq!(config.source, Some(PathBuf::from("/home/u/books/inner")));
    }

    #[test]
    fn config_home_is_appdata_on_windows() {
        let var = |name: &str| (name == "APPDATA").then(|| PathBuf::from("C:/AppData"));
        assert_eq!(
            config_home(true, var, Some(Path::new("C:/Users/u"))),
            Some(PathBuf::from("C:/AppData"))
        );
        assert_eq!(
            config_home(true, |_| None, Some(Path::new("C:/Users/u"))),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn config_home_is_xdg_then_dot_config() {
        let home = Some(Path::new("/home/u"));
        let xdg = |value: &'static str| {
            move |name: &str| (name == "XDG_CONFIG_HOME").then(|| PathBuf::from(value))
        };
        assert_eq!(
            config_home(false, xdg("/xdg"), home),
            Some(PathBuf::from("/xdg"))
        );
        // Relative values are ignored, as the XDG spec says.
        assert_eq!(
            config_home(false, xdg("xdg"), home),
            Some(PathBuf::from("/home/u/.config"))
        );
        assert_eq!(
            config_home(false, |_| None, home),
            Some(PathBuf::from("/home/u/.config"))
        );
        assert_eq!(config_home(false, |_| None, None), None);
    }

    #[test]
    fn user_config_is_under_config_home() {
        let fs = FakeFs::default().with("/xdg/okane/config.toml", "source = \"x\"");
        let env = Env {
            config_home: Some(PathBuf::from("/xdg")),
            ..env("/tmp")
        };
        let config = load(&env, &fs).unwrap();
        assert_eq!(config.source, Some(PathBuf::from("/xdg/okane/x")));
    }

    #[test]
    fn empty_okane_config_gives_default() {
        let fs = FakeFs::default()
            .with(USER, "source = \"user\"")
            .with("/home/u/books/okane.toml", "source = \"project\"");
        let env = Env {
            okane_config: OkaneConfigEnvVar::Empty,
            ..env("/home/u/books")
        };
        let config = load(&env, &fs).unwrap();
        assert_eq!(config.source, None);
    }

    #[test]
    fn okane_config_replaces_discovery() {
        let fs = FakeFs::default()
            .with(USER, "source = \"user\"")
            .with("/home/u/books/okane.toml", "source = \"project\"")
            .with(
                "/home/u/books/explicit.toml",
                "[report]\nexchange = \"JPY\"",
            );
        let env = Env {
            okane_config: OkaneConfigEnvVar::Set(PathBuf::from("explicit.toml")),
            ..env("/home/u/books")
        };
        let config = load(&env, &fs).unwrap();
        assert_eq!(config.source, None);
    }

    #[test]
    fn missing_okane_config_is_an_error() {
        let env = Env {
            okane_config: OkaneConfigEnvVar::Set(PathBuf::from("/nowhere.toml")),
            ..env("/tmp")
        };
        let err = load(&env, &FakeFs::default()).unwrap_err();
        assert_matches!(err, ConfigError::ExplicitNotFound(path) if path == Path::new("/nowhere.toml"));
    }

    #[test]
    fn unknown_key_is_an_error_naming_the_file() {
        let fs = FakeFs::default().with(USER, "[report]\nprice_db = \"x\"");
        let err = load(&env("/tmp"), &fs).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains(&user_display()), "{msg}");
        assert!(msg.contains("price_db"), "{msg}");
    }

    #[test]
    fn key_in_the_wrong_table_is_an_error() {
        let fs = FakeFs::default().with(USER, "[balance]\nsort = \"date\"");
        assert_matches!(load(&env("/tmp"), &fs), Err(ConfigError::Parse { .. }));
        let fs = FakeFs::default().with(USER, "[register]\nrules = \"x\"");
        assert_matches!(load(&env("/tmp"), &fs), Err(ConfigError::Parse { .. }));
    }

    #[test]
    fn unknown_enum_value_is_an_error() {
        let fs = FakeFs::default().with(USER, "[register]\nsort = \"amount\"");
        assert_matches!(load(&env("/tmp"), &fs), Err(ConfigError::Parse { .. }));
    }

    #[test]
    fn read_error_names_the_file() {
        struct Broken;
        impl ConfigFs for Broken {
            fn read(&self, _: &Path) -> io::Result<Option<String>> {
                Err(io::Error::other("broken"))
            }
        }
        let err = load(&env("/tmp"), &Broken).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("failed to read config file {}", user_display())
        );
    }

    #[test]
    fn import_ledger_falls_back_to_source() {
        let config = Config {
            source: Some(PathBuf::from("/main.ledger")),
            ..Config::default()
        };
        assert_eq!(config.import_ledger(), Some(Path::new("/main.ledger")));
        let config = Config {
            import: ImportTable {
                ledger: Some(PathBuf::from("/accounts.ledger")),
                ..ImportTable::default()
            },
            ..config
        };
        assert_eq!(config.import_ledger(), Some(Path::new("/accounts.ledger")));
    }

    #[test]
    fn tilde_without_home_is_kept_relative() {
        let resolver = PathResolver {
            base: Path::new("/base"),
            home: None,
        };
        assert_eq!(resolver.resolved(Path::new("~/x")), Path::new("/base/~/x"));
    }

    #[test]
    fn real_fs_reads_files_and_tells_missing_from_broken() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.toml");
        std::fs::write(&file, "source = 'x'").unwrap();

        assert_eq!(RealFs.read(&file).unwrap().as_deref(), Some("source = 'x'"));
        assert_eq!(RealFs.read(&dir.path().join("missing.toml")).unwrap(), None);
        // A directory exists but can't be read as a file.
        RealFs.read(dir.path()).unwrap_err();
    }
}
