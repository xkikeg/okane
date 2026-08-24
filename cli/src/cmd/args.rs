//! What the user typed: the structs clap parses the command line into.
//!
//! These say nothing about whether the flags make sense together. Each one
//! `build()`s the matching command in [`super`], which holds only valid states,
//! so the commands never re-check their flags while running.

use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Subcommand};

use okane_core::report::query;

use super::{
    AccountFilterMode, AccountsCmd, BalanceCmd, Command, EvalCmd, EvalOptions, FlattenCmd,
    FormatCmd, ImportCmd, ImportMode, InvalidFlagError, PrimitiveCmd, RegisterCmd, SortKey,
    TagsCmd, UiCmd,
};
use crate::build::CLAP_LONG_VERSION;
use crate::ui::migemo::MigemoCommand;

#[derive(clap::Parser, Debug)]
#[clap(about, version, author, long_version = CLAP_LONG_VERSION)]
#[command(infer_subcommands = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: CommandArgs,
}

impl Cli {
    /// The command to run, or why the given flags don't describe one.
    pub fn build(self) -> Result<Command, InvalidFlagError> {
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
    fn build(self) -> Result<Command, InvalidFlagError> {
        Ok(match self {
            CommandArgs::Import(args) => Command::Import(args.build()?),
            CommandArgs::Format(args) => Command::Format(args.build()),
            CommandArgs::Accounts(args) => Command::Accounts(args.build()),
            CommandArgs::Tags(args) => Command::Tags(args.build()),
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
    #[arg(
        short = 'r',
        long,
        value_name = "FILE",
        required_unless_present = "config"
    )]
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
    #[arg(long, value_name = "FILE", requires = "interactive")]
    ledger: Option<PathBuf>,

    /// Ledger file the reviewed transactions are appended to (interactive only).
    #[arg(short = 'o', long, value_name = "FILE", requires = "interactive")]
    output: Option<PathBuf>,

    source: PathBuf,
}

impl ImportArgs {
    fn build(self) -> Result<ImportCmd, InvalidFlagError> {
        let rules = match (self.rules, self.config) {
            (Some(rules), _) => rules,
            (None, Some(config)) => {
                eprintln!("warning: `--config` / `-c` is deprecated, use `--rules` / `-r` instead");
                config
            }
            (None, None) => return Err(InvalidFlagError("--rules is required".to_string())),
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

#[derive(Args, Debug)]
struct AccountsArgs {
    source: PathBuf,
}

impl AccountsArgs {
    fn build(self) -> AccountsCmd {
        AccountsCmd {
            source: self.source,
        }
    }
}

#[derive(Args, Debug)]
struct TagsArgs {
    /// Path to the Ledger file.
    source: PathBuf,

    /// Print each tag as `key: value`, instead of the key alone.
    ///
    /// Tags without a value, such as `; :tag1:tag2:`, are still printed as the bare key.
    #[arg(long)]
    values: bool,
}

impl TagsArgs {
    fn build(self) -> TagsCmd {
        TagsCmd {
            source: self.source,
            values: self.values,
        }
    }
}

#[derive(Args, Debug)]
struct BalanceArgs {
    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    /// Path to the Ledger file.
    source: PathBuf,

    /// [Optional] Accounts to report the balance.
    ///
    /// By default, each pattern is an unanchored regex matched against the account name.
    /// Use `--account-filter` to change the matching logic.
    ///
    /// If none are set, show all accounts.
    ///
    /// If any of them are non-existing accounts, those are simply skipped.
    account: Vec<String>,
}

impl BalanceArgs {
    fn build(self) -> Result<BalanceCmd, InvalidFlagError> {
        Ok(BalanceCmd {
            eval_options: self.eval_options.build()?,
            source: self.source,
            account: self.account,
        })
    }
}

#[derive(Args, Debug)]
struct UiArgs {
    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    /// Command running migemo, used to expand the search query.
    ///
    /// Given a migemo command, `/` and `C-s` search (of accounts on the
    /// balance screen, payees on the register) take a romaji query and match
    /// the Japanese it stands for: typing `ginkou` finds `資産:銀行`.
    /// Without it the query is used as the regex, as typed.
    ///
    /// The value is one shell-quoted command line, so quote it as a whole:
    /// `--migemo='cmigemo -q -d /usr/share/cmigemo/utf-8/migemo-dict'`.
    /// The command must read a query per line on stdin and write the regex it
    /// expands to on stdout, which is what `cmigemo` does.
    #[arg(long, value_name = "COMMAND")]
    migemo: Option<String>,

    /// Path to the Ledger file.
    source: PathBuf,
}

impl UiArgs {
    fn build(self) -> Result<UiCmd, InvalidFlagError> {
        let migemo = self
            .migemo
            .map(|command| MigemoCommand::parse(&command))
            .transpose()
            .map_err(|err| InvalidFlagError(err.to_string()))?;
        Ok(UiCmd {
            eval_options: self.eval_options.build()?,
            migemo,
            source: self.source,
        })
    }
}

#[derive(Args, Debug)]
struct RegisterArgs {
    #[command(flatten)]
    eval_options: EvalOptionsArgs,

    /// Sort order for the register rows.
    #[arg(long, value_enum, default_value_t)]
    sort: SortKey,

    /// Path to the Ledger file.
    source: PathBuf,

    /// [Optional] Accounts to get register.
    ///
    /// By default, each pattern is an unanchored regex matched against the
    /// account name. an account is shown when it matches any pattern.
    /// Use `--account-filter` to change the matching logic.
    ///
    /// If none are set, show all accounts.
    ///
    /// If any of them are non-existing accounts, those are simply skipped.
    account: Vec<String>,
}

impl RegisterArgs {
    fn build(self) -> Result<RegisterCmd, InvalidFlagError> {
        Ok(RegisterCmd {
            eval_options: self.eval_options.build()?,
            sort: self.sort,
            source: self.source,
            account: self.account,
        })
    }
}

/// The evaluation flags shared by `balance`, `register`, `ui` and
/// `primitive eval`.
#[derive(Args, Debug)]
struct EvalOptionsArgs {
    /// Path to the Price DB.
    #[arg(long)]
    price_db: Option<PathBuf>,

    /// Commodity used for the evaluation.
    ///
    /// When user specifies `--exchange=FOO`,
    /// all values in other commmodities are converted to FOO.
    #[arg(short = 'X', long)]
    exchange: Option<String>,

    /// Use historical rate for exchange.
    ///
    /// Option `--historical` to evaluate exchange rate at the date of transaction.
    /// This is useful for evaluating income and expense,
    /// while it's pointless for assets and liabilities.
    #[arg(long, default_value_t)]
    historical: bool,

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
    /// By default, `regex` mode is used.
    #[arg(long, value_enum, default_value_t)]
    account_filter: AccountFilterMode,
}

impl EvalOptionsArgs {
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
            account_filter: self.account_filter,
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

    /// Parses `okane <args>` and builds the command.
    fn build(args: &[&str]) -> Result<Command, InvalidFlagError> {
        Cli::try_parse_from(std::iter::once("okane").chain(args.iter().copied()))
            .expect("the flags must parse")
            .build()
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
        assert_eq!(err.to_string(), "invalid flag: --rules is required");
    }

    #[test]
    fn import_build_rejects_interactive_without_ledger_or_output() {
        assert!(import_args(true, false, true).build().is_err());
        assert!(import_args(true, true, false).build().is_err());
        assert!(import_args(true, false, false).build().is_err());
    }

    #[test]
    fn ui_migemo_is_parsed_when_building() {
        let cmd = build(&["ui", "--migemo=cmigemo -d 'my dict'", "a.ledger"]);
        let cmd = assert_matches!(cmd, Ok(Command::Ui(cmd)) => cmd);
        assert_eq!(
            cmd.migemo,
            Some(MigemoCommand::parse("cmigemo -d 'my dict'").unwrap())
        );

        let err = build(&["ui", "--migemo=cmigemo 'unbalanced", "a.ledger"]).unwrap_err();
        assert!(
            err.to_string()
                .starts_with("invalid flag: --migemo command"),
            "{err}"
        );
        let err = build(&["ui", "--migemo=", "a.ledger"]).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid flag: --migemo needs a command to run"
        );
    }

    #[test]
    fn eval_options_resolve_current_to_the_day_after_today() {
        let cmd = build(&["balance", "--today=2024-03-31", "--current", "a.ledger"]).unwrap();
        let cmd = assert_matches!(cmd, Command::Balance(cmd) => cmd);
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
        let err = build(&["ui", "--current", "--end=2024-02-01", "a.ledger"]).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid flag: --current and --end cannot be set simultaneously"
        );
    }

    #[test]
    fn eval_options_reject_current_on_the_last_representable_day() {
        let today = NaiveDate::MAX.to_string();
        let err = build(&["balance", "--today", &today, "--current", "a.ledger"]).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("invalid flag: cannot compute one day after today {today}")
        );
    }
}
