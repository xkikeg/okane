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

use serde::Deserialize;

use crate::cmd::{AccountFilterMode, SortKey};

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
    pub report: ReportTable,
    pub balance: ReportTable,
    pub register: RegisterTable,
    pub ui: ReportTable,
    pub import: ImportTable,
}

/// Flags shared by the report commands, as in `[report]` and `[balance]`.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ReportTable {
    pub price_db: Option<PathBuf>,
    pub exchange: Option<String>,
    pub historical: Option<bool>,
    pub account_filter: Option<AccountFilterMode>,
}

/// `[register]`: [`ReportTable`] plus the register only flags.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct RegisterTable {
    pub price_db: Option<PathBuf>,
    pub exchange: Option<String>,
    pub historical: Option<bool>,
    pub account_filter: Option<AccountFilterMode>,
    pub sort: Option<SortKey>,
}

/// `[import]`.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ImportTable {
    pub rules: Option<PathBuf>,
    /// Ledger for `--interactive`, falling back to the top-level `source`.
    pub ledger: Option<PathBuf>,
}

/// Sets each `None` field of `$lhs` from `$rhs`.
macro_rules! fill {
    ($lhs:expr, $rhs:expr; $($field:ident),+ $(,)?) => {
        $(
            if $lhs.$field.is_none() {
                $lhs.$field = $rhs.$field.clone();
            }
        )+
    };
}

impl ReportTable {
    /// Returns `self`, with the keys it lacks taken from `base`.
    pub fn or(mut self, base: &ReportTable) -> ReportTable {
        fill!(self, base; price_db, exchange, historical, account_filter);
        self
    }
}

impl RegisterTable {
    /// The keys shared with `[report]`.
    pub fn report(&self) -> ReportTable {
        ReportTable {
            price_db: self.price_db.clone(),
            exchange: self.exchange.clone(),
            historical: self.historical,
            account_filter: self.account_filter,
        }
    }
}

impl Config {
    /// Effective `[balance]`, falling back to `[report]`.
    pub fn balance(&self) -> ReportTable {
        self.balance.clone().or(&self.report)
    }

    /// Effective report keys of `[register]`, falling back to `[report]`.
    pub fn register(&self) -> ReportTable {
        self.register.report().or(&self.report)
    }

    /// Effective report keys of `[ui]`, falling back to `[report]`.
    pub fn ui(&self) -> ReportTable {
        self.ui.clone().or(&self.report)
    }

    /// Ledger for `import --interactive`.
    pub fn import_ledger(&self) -> Option<&Path> {
        self.import.ledger.as_deref().or(self.source.as_deref())
    }

    /// Returns `self`, with the keys it lacks taken from `base`.
    fn or(mut self, base: Config) -> Config {
        fill!(self, base; source);
        self.report = self.report.or(&base.report);
        self.balance = self.balance.or(&base.balance);
        fill!(self.register, base.register;
            price_db, exchange, historical, account_filter, sort);
        self.ui = self.ui.or(&base.ui);
        fill!(self.import, base.import; rules, ledger);
        self
    }

    /// Makes the paths absolute: relative ones are relative to `base`,
    /// the directory of the file, and `~` stands for `home`.
    fn resolve_paths(&mut self, base: &Path, home: Option<&Path>) {
        let paths = [
            &mut self.source,
            &mut self.report.price_db,
            &mut self.balance.price_db,
            &mut self.register.price_db,
            &mut self.ui.price_db,
            &mut self.import.rules,
            &mut self.import.ledger,
        ];
        for path in paths.into_iter().flatten() {
            *path = resolve_path(path, base, home);
        }
    }
}

fn resolve_path(path: &Path, base: &Path, home: Option<&Path>) -> PathBuf {
    let mut components = path.components();
    match (components.next(), home) {
        (Some(Component::Normal(first)), Some(home)) if first == "~" => {
            home.join(components.as_path())
        }
        _ => base.join(path),
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
    pub okane_config: Option<PathBuf>,
}

impl Env {
    /// Reads the environment of this process.
    pub fn from_process() -> io::Result<Self> {
        let var = |name: &str| {
            std::env::var_os(name)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        let home = std::env::home_dir().filter(|p| !p.as_os_str().is_empty());
        Ok(Env {
            cwd: std::env::current_dir()?,
            config_home: config_home(cfg!(windows), var, home.as_deref()),
            home,
            okane_config: var("OKANE_CONFIG"),
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
    if let Some(path) = &env.okane_config {
        let path = env.cwd.join(path);
        let content =
            read(fs, &path)?.ok_or_else(|| ConfigError::ExplicitNotFound(path.clone()))?;
        return parse(&path, &content, env);
    }
    let mut config = Config::default();
    if let Some(path) = env.user_config()
        && let Some(content) = read(fs, &path)?
    {
        config = parse(&path, &content, env)?;
    }
    if let Some((path, content)) = find_project(env, fs)? {
        config = parse(&path, &content, env)?.or(config);
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
    let base = path.parent().unwrap_or(Path::new(""));
    config.resolve_paths(base, env.home.as_deref());
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
            okane_config: None,
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
                historical = true
                account-filter = "exact"

                [balance]
                exchange = "CHF"

                [register]
                sort = "date"

                [ui]
                historical = false

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
                    historical: Some(true),
                    account_filter: Some(AccountFilterMode::Exact),
                },
                balance: ReportTable {
                    exchange: Some("CHF".to_owned()),
                    ..ReportTable::default()
                },
                register: RegisterTable {
                    sort: Some(SortKey::Date),
                    ..RegisterTable::default()
                },
                ui: ReportTable {
                    historical: Some(false),
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
    fn command_tables_fall_back_to_report() {
        let fs = FakeFs::default().with(
            USER,
            indoc! {r#"
                [report]
                exchange = "JPY"
                historical = true
                [balance]
                exchange = "CHF"
                [register]
                historical = false
            "#},
        );
        let config = load(&env("/tmp"), &fs).unwrap();
        assert_eq!(config.balance().exchange.as_deref(), Some("CHF"));
        assert_eq!(config.balance().historical, Some(true));
        assert_eq!(config.register().exchange.as_deref(), Some("JPY"));
        assert_eq!(config.register().historical, Some(false));
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
    fn okane_config_replaces_discovery() {
        let fs = FakeFs::default()
            .with(USER, "source = \"user\"")
            .with("/home/u/books/okane.toml", "source = \"project\"")
            .with(
                "/home/u/books/explicit.toml",
                "[report]\nexchange = \"JPY\"",
            );
        let env = Env {
            okane_config: Some(PathBuf::from("explicit.toml")),
            ..env("/home/u/books")
        };
        let config = load(&env, &fs).unwrap();
        assert_eq!(config.source, None);
    }

    #[test]
    fn missing_okane_config_is_an_error() {
        let env = Env {
            okane_config: Some(PathBuf::from("/nowhere.toml")),
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
        let path = resolve_path(Path::new("~/x"), Path::new("/base"), None);
        assert_eq!(path, PathBuf::from("/base/~/x"));
    }
}
