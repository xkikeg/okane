//! What the user typed: the structs clap parses the command line into.
//!
//! These say nothing about whether the flags make sense together. Each one
//! `build()`s the matching command in [`super`], which holds only valid states,
//! so the commands never re-check their flags while running. Given a config
//! file, [`Cli::build`] first fills in what the command line left out from it
//! (the private `with_config()` methods); flags given on the command line
//! always win.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::{Args, Subcommand};

use okane_core::report::query;

use super::{
    AccountFilterMode, AccountsCmd, BalanceCmd, Command, EvalCmd, EvalOptions, FlattenCmd,
    FormatCmd, ImportCmd, ImportMode, InvalidFlagError, PrimitiveCmd, RegisterCmd, SortKey,
    TagsCmd, UiCmd,
};
use crate::build::CLAP_LONG_VERSION;
use crate::config::{Config, ReportTable};

#[derive(clap::Parser, Debug)]
#[clap(about, version, author, long_version = CLAP_LONG_VERSION)]
#[command(infer_subcommands = true)]
pub struct Cli {
    /// Don't read any config file.
    ///
    /// Without it, flag defaults are read from the user config
    /// (`$XDG_CONFIG_HOME/okane/config.toml`) and the nearest `okane.toml`,
    /// or only from `$OKANE_CONFIG` when it is set.
    #[arg(long)]
    no_config: bool,

    #[clap(subcommand)]
    command: CommandArgs,
}

impl Cli {
    /// Whether config files should be skipped.
    pub fn no_config(&self) -> bool {
        self.no_config
    }

    /// The command to run, or why the given flags don't describe one.
    ///
    /// The flags not given on the command line are taken from `config`, if
    /// any; pass `None` for [`Self::no_config`].
    pub fn build(mut self, config: Option<&Config>) -> Result<Command, InvalidFlagError> {
        if let Some(config) = config {
            self.command.with_config(config);
        }
        self.command.build()
    }
}

#[derive(Subcommand, Debug)]
enum CommandArgs {
    /// Import other format into ledger.
    Import(ImportArgs),
    /// Format the given file (in future it'll work without file arg)
    Format(FormatArgs),
    /// List all accounts in the file.
    Accounts(AccountsArgs),
    /// List all tags in the file.
    Tags(TagsArgs),
    /// Gives balance report.
    Balance(BalanceArgs),
    /// Gives register report.
    Register(RegisterArgs),
    /// Open an interactive terminal UI showing the balance report.
    Ui(UiArgs),
    /// Primitive is a set of commands which are primitive and suitable for debugging.
    Primitive(PrimitiveArgs),
}

impl CommandArgs {
    fn with_config(&mut self, config: &Config) {
        match self {
            CommandArgs::Import(args) => args.with_config(config),
            CommandArgs::Accounts(args) => args.file.with_config(config),
            CommandArgs::Tags(args) => args.file.with_config(config),
            CommandArgs::Balance(args) => args.with_config(config),
            CommandArgs::Register(args) => args.with_config(config),
            CommandArgs::Ui(args) => args.with_config(config),
            CommandArgs::Format(_) | CommandArgs::Primitive(_) => (),
        }
    }

    fn build(self) -> Result<Command, InvalidFlagError> {
        Ok(match self {
            CommandArgs::Import(args) => Command::Import(args.build()?),
            CommandArgs::Format(args) => Command::Format(args.build()),
            CommandArgs::Accounts(args) => Command::Accounts(args.build()?),
            CommandArgs::Tags(args) => Command::Tags(args.build()?),
            CommandArgs::Balance(args) => Command::Balance(args.build()?),
            CommandArgs::Register(args) => Command::Register(args.build()?),
            CommandArgs::Ui(args) => Command::Ui(args.build()?),
            CommandArgs::Primitive(args) => Command::Primitive(args.build()?),
        })
    }
}

#[derive(Args, Debug)]
pub(super) struct ImportArgs {
    /// YAML file with the rules that say how to import each source file.
    ///
    /// Defaults to `rules` in the `[import]` table of the config file.
    #[arg(short = 'r', long, value_name = "FILE")]
    rules: Option<PathBuf>,

    /// Deprecated name of `--rules`, kept so that existing scripts still work.
    #[arg(
        short = 'c',
        long,
        value_name = "FILE",
        hide = true,
        conflicts_with = "rules"
    )]
    config: Option<PathBuf>,

    /// Review the imported transactions in an interactive TUI before writing.
    #[arg(long)]
    interactive: bool,

    /// Ledger file whose accounts feed the autocomplete (interactive only).
    ///
    /// Defaults to `ledger` in the `[import]` table of the config file,
    /// then to the top-level `source`.
    #[arg(long, value_name = "FILE", requires = "interactive")]
    ledger: Option<PathBuf>,

    /// Ledger file the reviewed transactions are appended to (interactive only).
    #[arg(short = 'o', long, value_name = "FILE", requires = "interactive")]
    output: Option<PathBuf>,

    source: PathBuf,
}

impl ImportArgs {
    fn with_config(&mut self, config: &Config) {
        if self.rules.is_none() && self.config.is_none() {
            self.rules = config.import.rules.clone();
        }
        if self.interactive && self.ledger.is_none() {
            self.ledger = config.import_ledger().map(Path::to_owned);
        }
    }

    fn build(self) -> Result<ImportCmd, InvalidFlagError> {
        let rules = match (self.rules, self.config) {
            (Some(rules), _) => rules,
            (None, Some(config)) => {
                eprintln!("warning: `--config` / `-c` is deprecated, use `--rules` / `-r` instead");
                config
            }
            (None, None) => {
                return Err(InvalidFlagError(
                    "--rules is required, unless `rules` is set in the `[import]` table of the config file"
                        .to_string(),
                ));
            }
        };
        let mode = match (self.interactive, self.ledger, self.output) {
            (false, _, _) => ImportMode::Batch,
            (true, Some(ledger), Some(output)) => ImportMode::Interactive { ledger, output },
            (true, _, _) => {
                return Err(InvalidFlagError(
                    "--interactive requires both --ledger and --output".to_string(),
                ));
            }
        };
        Ok(ImportCmd {
            rules,
            mode,
            source: self.source,
        })
    }
}

#[derive(Args, Debug)]
pub(super) struct FormatArgs {
    source: PathBuf,
}

impl FormatArgs {
    fn build(self) -> FormatCmd {
        FormatCmd {
            source: self.source,
        }
    }
}

#[derive(Args, Debug)]
struct PrimitiveArgs {
    #[command(subcommand)]
    command: PrimitiveCommandArgs,
}

impl PrimitiveArgs {
    fn build(self) -> Result<PrimitiveCmd, InvalidFlagError> {
        Ok(match self.command {
            PrimitiveCommandArgs::Format(args) => PrimitiveCmd::Format(args.build()),
            PrimitiveCommandArgs::Flatten(args) => PrimitiveCmd::Flatten(args.build()),
            PrimitiveCommandArgs::Eval(args) => PrimitiveCmd::Eval(args.build()?),
        })
    }
}

#[derive(Subcommand, Debug)]
enum PrimitiveCommandArgs {
    /// Format the given one ledger file, to stdout.
    Format(FormatArgs),
    /// Read the given one ledger file, recursively resolves include directives and print to stdout.
    Flatten(FlattenArgs),
    /// Evaluates the given value under given condition.
    Eval(EvalArgs),
}

#[derive(Args, Debug)]
struct FlattenArgs {
    source: PathBuf,
}

impl FlattenArgs {
    fn build(self) -> FlattenCmd {
        FlattenCmd {
            source: self.source,
        }
    }
}

#[derive(Args, Debug)]
struct EvalArgs {
    /// Date of the price.
    #[arg(long)]
    date: NaiveDate,

    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    /// source of the Ledger file.
    #[arg(short = 'f', long = "file")]
    source: PathBuf,

    /// expression to evaluate.
    expression: Vec<String>,
}

impl EvalArgs {
    fn build(self) -> Result<EvalCmd, InvalidFlagError> {
        Ok(EvalCmd {
            date: self.date,
            eval_options: self.eval_options.build()?,
            source: self.source,
            expression: self.expression,
        })
    }
}

/// Ledger file of the commands that take nothing else positional.
#[derive(Args, Debug)]
struct LedgerFileArgs {
    /// Path to the Ledger file.
    ///
    /// Defaults to `source` in the config file.
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<PathBuf>,

    /// Path to the Ledger file, same as `--file`.
    #[arg(value_name = "SOURCE", conflicts_with = "file")]
    source: Option<PathBuf>,
}

impl LedgerFileArgs {
    fn with_config(&mut self, config: &Config) {
        if self.file.is_none() && self.source.is_none() {
            self.file = config.source.clone();
        }
    }

    fn build(self) -> Result<PathBuf, InvalidFlagError> {
        self.file.or(self.source).ok_or_else(missing_ledger_file)
    }
}

fn missing_ledger_file() -> InvalidFlagError {
    InvalidFlagError(
        "no Ledger file given; pass it with --file, or set `source` in the config file".to_string(),
    )
}

/// The Ledger file of `balance` and `register`, whose positional arguments
/// are account patterns.
///
/// Without `--file` (or `source` in the config file), the first positional
/// argument is taken as the file instead, as before `--file` existed.
fn report_file(
    file: Option<PathBuf>,
    account: &mut Vec<String>,
) -> Result<PathBuf, InvalidFlagError> {
    if let Some(file) = file {
        return Ok(file);
    }
    if account.is_empty() {
        return Err(missing_ledger_file());
    }
    eprintln!(
        "warning: passing the Ledger file as the first argument is deprecated, use `--file` / `-f` instead"
    );
    Ok(PathBuf::from(account.remove(0)))
}

#[derive(Args, Debug)]
struct AccountsArgs {
    #[command(flatten)]
    file: LedgerFileArgs,
}

impl AccountsArgs {
    fn build(self) -> Result<AccountsCmd, InvalidFlagError> {
        Ok(AccountsCmd {
            source: self.file.build()?,
        })
    }
}

#[derive(Args, Debug)]
struct TagsArgs {
    #[command(flatten)]
    file: LedgerFileArgs,

    /// Print each tag as `key: value`, instead of the key alone.
    ///
    /// Tags without a value, such as `; :tag1:tag2:`, are still printed as the bare key.
    #[arg(long)]
    values: bool,
}

impl TagsArgs {
    fn build(self) -> Result<TagsCmd, InvalidFlagError> {
        Ok(TagsCmd {
            source: self.file.build()?,
            values: self.values,
        })
    }
}

#[derive(Args, Debug)]
struct BalanceArgs {
    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    /// Path to the Ledger file.
    ///
    /// Defaults to `source` in the config file.
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<PathBuf>,

    /// [Optional] Accounts to report the balance.
    ///
    /// By default, each pattern is an unanchored regex matched against the account name.
    /// Use `--account-filter` to change the matching logic.
    ///
    /// If none are set, show all accounts.
    ///
    /// If any of them are non-existing accounts, those are simply skipped.
    ///
    /// Without `--file` or `source` in the config file, the first one is
    /// taken as the Ledger file instead, which is deprecated.
    #[arg(value_name = "ACCOUNT")]
    account: Vec<String>,
}

impl BalanceArgs {
    fn with_config(&mut self, config: &Config) {
        self.eval_options.with_config(&config.balance());
        if self.file.is_none() {
            self.file = config.source.clone();
        }
    }

    fn build(mut self) -> Result<BalanceCmd, InvalidFlagError> {
        Ok(BalanceCmd {
            eval_options: self.eval_options.build()?,
            source: report_file(self.file, &mut self.account)?,
            account: self.account,
        })
    }
}

#[derive(Args, Debug)]
struct UiArgs {
    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    #[command(flatten)]
    file: LedgerFileArgs,
}

impl UiArgs {
    fn with_config(&mut self, config: &Config) {
        self.eval_options.with_config(&config.ui());
        self.file.with_config(config);
    }

    fn build(self) -> Result<UiCmd, InvalidFlagError> {
        Ok(UiCmd {
            eval_options: self.eval_options.build()?,
            source: self.file.build()?,
        })
    }
}

#[derive(Args, Debug)]
struct RegisterArgs {
    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    /// Sort order for the register rows.
    ///
    /// Defaults to `sort` in the `[register]` table of the config file,
    /// then to `original`.
    #[arg(long, value_enum)]
    sort: Option<SortKey>,

    /// Path to the Ledger file.
    ///
    /// Defaults to `source` in the config file.
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<PathBuf>,

    /// [Optional] Accounts to get register.
    ///
    /// By default, each pattern is an unanchored regex matched against the
    /// account name. an account is shown when it matches any pattern.
    /// Use `--account-filter` to change the matching logic.
    ///
    /// If none are set, show all accounts.
    ///
    /// If any of them are non-existing accounts, those are simply skipped.
    ///
    /// Without `--file` or `source` in the config file, the first one is
    /// taken as the Ledger file instead, which is deprecated.
    #[arg(value_name = "ACCOUNT")]
    account: Vec<String>,
}

impl RegisterArgs {
    fn with_config(&mut self, config: &Config) {
        self.eval_options.with_config(&config.register());
        if self.sort.is_none() {
            self.sort = config.register.sort;
        }
        if self.file.is_none() {
            self.file = config.source.clone();
        }
    }

    fn build(mut self) -> Result<RegisterCmd, InvalidFlagError> {
        Ok(RegisterCmd {
            eval_options: self.eval_options.build()?,
            sort: self.sort.unwrap_or_default(),
            source: report_file(self.file, &mut self.account)?,
            account: self.account,
        })
    }
}

/// The evaluation flags shared by `balance`, `register`, `ui` and
/// `primitive eval`.
#[derive(Args, Debug)]
struct EvalOptionsArgs {
    /// Path to the Price DB.
    ///
    /// Defaults to `price-db` in the config file.
    #[arg(long)]
    price_db: Option<PathBuf>,

    /// Commodity used for the evaluation.
    ///
    /// When user specifies `--exchange=FOO`,
    /// all values in other commmodities are converted to FOO.
    ///
    /// Defaults to `exchange` in the config file.
    #[arg(short = 'X', long)]
    exchange: Option<String>,

    /// Use historical rate for exchange.
    ///
    /// Option `--historical` to evaluate exchange rate at the date of transaction.
    /// This is useful for evaluating income and expense,
    /// while it's pointless for assets and liabilities.
    ///
    /// Defaults to `historical` in the config file.
    #[arg(long, overrides_with = "no_historical")]
    historical: bool,

    /// Use the rate at `--today` for exchange, even if the config file sets `historical`.
    #[arg(long, overrides_with = "historical")]
    no_historical: bool,

    /// Today's date in YYYY-mm-dd format.
    ///
    /// By default it points to the current local date.
    /// This value is used in `--current`, as well as currency conversion.
    #[arg(long, visible_alias("now"),default_value_t = chrono::Local::now().date_naive())]
    today: NaiveDate,

    /// Beginning of date range (inclusive).
    ///
    /// If specified, only transaction with the date equals/after `--start` is considered.
    #[arg(long, visible_alias("begin"))]
    start: Option<NaiveDate>,

    /// End of date range (exclusive).
    ///
    /// If specified, only transaction with the date before `--end` is considered.
    #[arg(long)]
    end: Option<NaiveDate>,

    /// If specified, sets the end date to `--today`.
    ///
    /// If this is specified with `--end`, it causes an error.
    #[arg(long, default_value_t)]
    current: bool,

    /// Controls the account filter mode.
    ///
    /// Defaults to `account-filter` in the config file, then to `regex`.
    #[arg(long, value_enum)]
    account_filter: Option<AccountFilterMode>,
}

impl EvalOptionsArgs {
    /// Fills the flags not given on the command line from `table`.
    fn with_config(&mut self, table: &ReportTable) {
        if self.price_db.is_none() {
            self.price_db = table.price_db.clone();
        }
        if self.exchange.is_none() {
            self.exchange = table.exchange.clone();
        }
        if !self.historical && !self.no_historical {
            self.historical = table.historical.unwrap_or(false);
        }
        if self.account_filter.is_none() {
            self.account_filter = table.account_filter;
        }
    }

    /// Resolves `--current` into the end of the date range, which is the only
    /// form the commands use.
    fn build(self) -> Result<EvalOptions, InvalidFlagError> {
        let end = match (self.current, self.end) {
            (true, Some(_)) => {
                return Err(InvalidFlagError(
                    "--current and --end cannot be set simultaneously".to_string(),
                ));
            }
            (true, None) => Some(self.today.succ_opt().ok_or_else(|| {
                InvalidFlagError(format!("cannot compute one day after today {}", self.today))
            })?),
            (false, end) => end,
        };
        Ok(EvalOptions {
            price_db: self.price_db,
            exchange: self.exchange,
            historical: self.historical,
            today: self.today,
            date_range: query::DateRange {
                start: self.start,
                end,
            },
            account_filter: self.account_filter.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use clap::Parser as _;

    #[test]
    fn verify_command() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("okane").chain(args.iter().copied()))
            .expect("the flags must parse")
    }

    /// Parses `okane <args>` and builds the command with `config`.
    fn build_with(args: &[&str], config: &Config) -> Result<Command, InvalidFlagError> {
        parse(args).build(Some(config))
    }

    /// Parses `okane <args>` and builds the command, without a config file.
    fn build(args: &[&str]) -> Result<Command, InvalidFlagError> {
        parse(args).build(None)
    }

    fn balance(cmd: Result<Command, InvalidFlagError>) -> BalanceCmd {
        assert_matches!(cmd, Ok(Command::Balance(cmd)) => cmd)
    }

    /// A config setting `historical` and `account-filter` for every report.
    fn report_config() -> Config {
        Config {
            report: ReportTable {
                historical: Some(true),
                account_filter: Some(AccountFilterMode::Exact),
                ..ReportTable::default()
            },
            ..Config::default()
        }
    }

    #[test]
    fn config_fills_flags_not_given() {
        let cmd = balance(build_with(&["balance", "-f", "x.ledger"], &report_config()));
        assert!(cmd.eval_options.historical);
        assert_eq!(cmd.eval_options.account_filter, AccountFilterMode::Exact);
    }

    #[test]
    fn flags_win_over_config() {
        let cmd = balance(build_with(
            &[
                "balance",
                "-f",
                "x.ledger",
                "--no-historical",
                "--account-filter=regex",
            ],
            &report_config(),
        ));
        assert!(!cmd.eval_options.historical);
        assert_eq!(cmd.eval_options.account_filter, AccountFilterMode::Regex);
    }

    #[test]
    fn last_of_historical_and_no_historical_wins() {
        let config = report_config();
        let cmd = balance(build_with(
            &[
                "balance",
                "-f",
                "x.ledger",
                "--historical",
                "--no-historical",
            ],
            &config,
        ));
        assert!(!cmd.eval_options.historical);
        let cmd = balance(build_with(
            &[
                "balance",
                "-f",
                "x.ledger",
                "--no-historical",
                "--historical",
            ],
            &config,
        ));
        assert!(cmd.eval_options.historical);
    }

    #[test]
    fn report_file_comes_from_file_flag_then_config_then_first_positional() {
        let config = Config {
            source: Some(PathBuf::from("config.ledger")),
            ..Config::default()
        };
        let cmd = balance(build_with(
            &["balance", "-f", "x.ledger", "Assets"],
            &config,
        ));
        assert_eq!(cmd.source, PathBuf::from("x.ledger"));
        assert_eq!(cmd.account, ["Assets"]);

        let cmd = balance(build_with(&["balance", "Assets"], &config));
        assert_eq!(cmd.source, PathBuf::from("config.ledger"));
        assert_eq!(cmd.account, ["Assets"]);

        let cmd = balance(build(&["balance", "legacy.ledger", "Assets"]));
        assert_eq!(cmd.source, PathBuf::from("legacy.ledger"));
        assert_eq!(cmd.account, ["Assets"]);
    }

    #[test]
    fn missing_ledger_file_is_a_flag_error() {
        let expected = "invalid flag: no Ledger file given; pass it with --file, or set `source` in the config file";
        assert_eq!(build(&["balance"]).unwrap_err().to_string(), expected);
        assert_eq!(build(&["accounts"]).unwrap_err().to_string(), expected);
    }

    #[test]
    fn register_sort_comes_from_config_then_defaults_to_original() {
        let config = Config {
            register: crate::config::RegisterTable {
                sort: Some(SortKey::Date),
                ..Default::default()
            },
            ..Config::default()
        };
        let cmd = build_with(&["register", "-f", "x.ledger"], &config);
        let cmd = assert_matches!(cmd, Ok(Command::Register(cmd)) => cmd);
        assert_eq!(cmd.sort, SortKey::Date);
        let cmd = build(&["register", "-f", "x.ledger"]);
        let cmd = assert_matches!(cmd, Ok(Command::Register(cmd)) => cmd);
        assert_eq!(cmd.sort, SortKey::Original);
    }

    fn import_args(interactive: bool, ledger: bool, output: bool) -> ImportArgs {
        ImportArgs {
            rules: Some(PathBuf::from("rules.yml")),
            config: None,
            interactive,
            ledger: ledger.then(|| PathBuf::from("root.ledger")),
            output: output.then(|| PathBuf::from("out.ledger")),
            source: PathBuf::from("source.csv"),
        }
    }

    #[test]
    fn import_build_accepts_batch_mode() {
        let cmd = import_args(false, false, false).build().unwrap();
        assert_eq!(cmd.rules, PathBuf::from("rules.yml"));
        assert_matches!(cmd.mode, ImportMode::Batch);
    }

    #[test]
    fn import_build_accepts_interactive_with_all_flags() {
        let cmd = import_args(true, true, true).build().unwrap();
        assert_matches!(
            cmd.mode,
            ImportMode::Interactive { ledger, output }
                if ledger.as_os_str() == "root.ledger" && output.as_os_str() == "out.ledger"
        );
    }

    #[test]
    fn import_build_takes_rules_from_deprecated_config() {
        let args = ImportArgs {
            rules: None,
            config: Some(PathBuf::from("old.yml")),
            ..import_args(false, false, false)
        };
        assert_eq!(args.build().unwrap().rules, PathBuf::from("old.yml"));
    }

    #[test]
    fn import_build_rejects_missing_rules() {
        let args = ImportArgs {
            rules: None,
            ..import_args(false, false, false)
        };
        let err = args.build().unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid flag: --rules is required, unless `rules` is set in the `[import]` table of the config file"
        );
    }

    #[test]
    fn import_build_rejects_interactive_without_ledger_or_output() {
        assert!(import_args(true, false, true).build().is_err());
        assert!(import_args(true, true, false).build().is_err());
        assert!(import_args(true, false, false).build().is_err());
    }

    #[test]
    fn eval_options_resolve_current_to_the_day_after_today() {
        let cmd = balance(build(&[
            "balance",
            "--today=2024-03-31",
            "--current",
            "-f",
            "a.ledger",
        ]));
        assert_eq!(
            cmd.eval_options.date_range.end,
            NaiveDate::from_ymd_opt(2024, 4, 1)
        );
    }

    #[test]
    fn eval_options_keep_end_without_current() {
        let cmd = build(&[
            "register",
            "--start=2024-01-01",
            "--end=2024-02-01",
            "-f",
            "a.ledger",
        ]);
        let cmd = assert_matches!(cmd, Ok(Command::Register(cmd)) => cmd);
        assert_eq!(
            cmd.eval_options.date_range.start,
            NaiveDate::from_ymd_opt(2024, 1, 1)
        );
        assert_eq!(
            cmd.eval_options.date_range.end,
            NaiveDate::from_ymd_opt(2024, 2, 1)
        );
    }

    #[test]
    fn eval_options_reject_current_with_end() {
        let err = build(&["ui", "--current", "--end=2024-02-01", "-f", "a.ledger"]).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid flag: --current and --end cannot be set simultaneously"
        );
    }

    #[test]
    fn eval_options_reject_current_on_the_last_representable_day() {
        let today = NaiveDate::MAX.to_string();
        let err =
            build(&["balance", "--today", &today, "--current", "-f", "a.ledger"]).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("invalid flag: cannot compute one day after today {today}")
        );
    }
}
