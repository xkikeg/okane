use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Context as _;
use bumpalo::Bump;
use chrono::NaiveDate;
use clap::{Args, Subcommand};

use lender::FallibleLender;
use okane_core::report::query;
use okane_core::syntax::display::DisplayContext;
use okane_core::syntax::plain::LedgerEntry;
use okane_core::{load, report};

use crate::build::CLAP_LONG_VERSION;
use crate::format;
use crate::import;
use crate::ui;

#[derive(thiserror::Error, Debug)]
#[error("invalid flag: {0}")]
pub struct InvalidFlagError(String);

#[derive(clap::Parser, Debug)]
#[clap(about, version, author, long_version = CLAP_LONG_VERSION)]
#[command(infer_subcommands = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

impl Cli {
    pub fn validate(&self) -> Result<(), InvalidFlagError> {
        self.command.validate()
    }

    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        self.command.run(w)
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Import other format into ledger.
    Import(ImportCmd),
    /// Format the given file (in future it'll work without file arg)
    Format(FormatCmd),
    /// List all accounts in the file.
    Accounts(AccountsCmd),
    /// Gives balance report.
    Balance(BalanceCmd),
    /// Gives register report.
    Register(RegisterCmd),
    /// Open an interactive terminal UI showing the balance report.
    Ui(UiCmd),
    /// Primitive is a set of commands which are primitive and suitable for debugging.
    Primitive(Primitives),
}

impl Command {
    fn validate(&self) -> Result<(), InvalidFlagError> {
        match self {
            Command::Import(cmd) => cmd.validate(),
            Command::Balance(cmd) => cmd.validate(),
            Command::Ui(cmd) => cmd.validate(),
            _ => Ok(()),
        }
    }

    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Command::Import(cmd) => cmd.run(w),
            Command::Format(cmd) => cmd.run(w),
            Command::Accounts(cmd) => cmd.run(w),
            Command::Balance(cmd) => cmd.run(w),
            Command::Register(cmd) => cmd.run(w),
            Command::Ui(cmd) => cmd.run(),
            Command::Primitive(cmd) => cmd.run(w),
        }
    }
}

#[derive(Args, Debug)]
pub struct ImportCmd {
    #[arg(short, long, value_name = "FILE")]
    pub config: std::path::PathBuf,

    /// Review the imported transactions in an interactive TUI before writing.
    #[arg(long)]
    pub interactive: bool,

    /// Ledger file whose accounts feed the autocomplete (interactive only).
    #[arg(long, value_name = "FILE", requires = "interactive")]
    pub ledger: Option<std::path::PathBuf>,

    /// Ledger file the reviewed transactions are appended to (interactive only).
    #[arg(short = 'o', long, value_name = "FILE", requires = "interactive")]
    pub output: Option<std::path::PathBuf>,

    pub source: std::path::PathBuf,
}

impl ImportCmd {
    fn validate(&self) -> Result<(), InvalidFlagError> {
        if self.interactive && (self.ledger.is_none() || self.output.is_none()) {
            return Err(InvalidFlagError(
                "--interactive requires both --ledger and --output".to_string(),
            ));
        }
        Ok(())
    }

    pub fn run<W>(&self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        if self.interactive {
            return self.run_interactive();
        }
        let importer = import::Importer::new(&self.config)?;
        importer.import_and_write(&self.source, w)?;
        Ok(())
    }

    fn run_interactive(&self) -> anyhow::Result<()> {
        let ledger_path = self.ledger.as_ref().expect("checked by validate()");
        let output_path = self.output.as_ref().expect("checked by validate()");
        let importer = import::Importer::new(&self.config)?;
        let (header, mut txns) = importer.import(&self.source)?;
        if txns.is_empty() {
            eprintln!("no transactions found in {}", self.source.display());
            return Ok(());
        }
        let accounts: Vec<String> = {
            let arena = Bump::new();
            let mut ctx = report::ReportContext::new(&arena);
            report::accounts(&mut ctx, load::new_loader(ledger_path.clone()))
                .with_context(|| format!("failed to load ledger {}", ledger_path.display()))?
                .iter()
                .map(|account| account.as_str().to_owned())
                .collect()
        };
        // accounts are already sorted.
        // Render every transaction up front: review items need the previews,
        // and any conversion error surfaces here, before entering raw mode.
        let items: Vec<ui::import::ReviewItem> = txns
            .iter()
            .map(|txn| {
                let preview = header.render_transaction(txn)?;
                Ok(ui::import::ReviewItem::new(
                    txn.review_kind(),
                    preview,
                    txn.date(),
                    txn.payee().to_owned(),
                    format!("{} {}", txn.amount().value, txn.amount().commodity),
                ))
            })
            .collect::<Result<_, import::ImportError>>()?;
        let mut app = ui::import::ReviewApp::new(
            self.source.display().to_string(),
            output_path.display().to_string(),
            items,
            accounts,
        );
        let outcome = ui::import::run_review(&mut app, &header, &mut txns)
            .context("failed to run review TUI")?;
        match outcome {
            ui::import::SessionOutcome::Abort => {
                eprintln!("import aborted; nothing written");
            }
            ui::import::SessionOutcome::Write => {
                let rendered: Vec<String> = txns
                    .iter()
                    .map(|txn| header.render_transaction(txn))
                    .collect::<Result<_, _>>()?;
                import::append_transactions(output_path, &rendered)
                    .with_context(|| format!("failed to append to {}", output_path.display()))?;
                eprintln!(
                    "appended {} transaction(s) to {}",
                    rendered.len(),
                    output_path.display()
                );
            }
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct FormatCmd {
    pub source: std::path::PathBuf,
}

impl FormatCmd {
    pub fn run<W>(&self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let mut r = BufReader::new(
            File::open(&self.source)
                .with_context(|| format!("failed to open {}", self.source.display()))?,
        );
        format::format(&mut r, w)?;
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct Primitives {
    #[command(subcommand)]
    command: PrimitiveCmd,
}

impl Primitives {
    fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        self.command.run(w)
    }
}

#[derive(Subcommand, Debug)]
enum PrimitiveCmd {
    /// Format the given one ledger file, to stdout.
    Format(FormatCmd),
    /// Read the given one ledger file, recursively resolves include directives and print to stdout.
    Flatten(FlattenCmd),
    /// Evaluates the given value under given condition.
    Eval(EvalCmd),
}

impl PrimitiveCmd {
    fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            PrimitiveCmd::Format(cmd) => cmd.run(w),
            PrimitiveCmd::Flatten(cmd) => cmd.run(w),
            PrimitiveCmd::Eval(cmd) => cmd.run(w),
        }
    }
}

#[derive(Args, Debug)]
struct FlattenCmd {
    pub source: std::path::PathBuf,
}

impl FlattenCmd {
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        // `Loader::load` constrains the callback's error to `E: From<LoadError>`,
        // so neither `io::Error` nor `anyhow::Error` fits — we use a tiny local
        // wrapper just to bridge the two.
        #[derive(thiserror::Error, Debug)]
        enum FlattenError {
            #[error(transparent)]
            Load(#[from] load::LoadError),
            #[error("failed to write flattened entry")]
            Write(#[from] std::io::Error),
        }

        // TODO: Pick DisplayContext from load results.
        let ctx = DisplayContext::default();
        load::new_loader(self.source).load(
            |_path, _ctx, entry: &LedgerEntry| -> Result<(), FlattenError> {
                writeln!(w, "{}", ctx.as_display(entry))?;
                Ok(())
            },
        )?;
        Ok(())
    }
}

#[derive(Args, Debug)]
struct EvalCmd {
    /// Date of the price.
    #[arg(long)]
    pub date: NaiveDate,

    #[command(flatten)]
    eval_options: EvalOptions,

    /// source of the Ledger file.
    #[arg(short = 'f', long = "file")]
    pub source: std::path::PathBuf,

    /// expression to evaluate.
    pub expression: Vec<String>,
}

impl EvalCmd {
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            load::new_loader(self.source),
            &self.eval_options.to_process_options(),
        )?;
        let mut expression: String = '('.to_string();
        for term in &self.expression {
            expression.push_str(term);
            expression.push(' ');
        }
        expression.push(')');
        let result = ledger.eval(
            &ctx,
            &expression,
            &query::EvalContext {
                date: self.date,
                exchange: self.eval_options.exchange,
            },
        )?;
        writeln!(w, "{}", result.as_inline_display(&ctx))?;
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AccountsCmd {
    pub source: std::path::PathBuf,
}

impl AccountsCmd {
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let accounts = report::accounts(&mut ctx, load::new_loader(self.source))?;
        for acc in accounts.iter() {
            writeln!(w, "{}", acc.as_str())?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct BalanceCmd {
    #[command(flatten)]
    eval_options: EvalOptions,

    /// Path to the Ledger file.
    source: std::path::PathBuf,

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

impl BalanceCmd {
    fn validate(&self) -> Result<(), InvalidFlagError> {
        self.eval_options.validate()?;
        Ok(())
    }
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            load::new_loader(self.source),
            &self.eval_options.to_process_options(),
        )?;
        let account = self
            .eval_options
            .create_account_filter(&ctx, self.account.as_slice())
            .context("failed to create regex for account filter")?;
        let query = query::BalanceQuery {
            account,
            conversion: self.eval_options.to_conversion(&ctx)?,
            date_range: self.eval_options.to_date_range()?,
        };
        for (account, amount) in ledger.balance(&ctx, &query)?.into_owned().into_vec() {
            writeln!(
                w,
                "{}: {}",
                account.as_str(),
                amount.as_inline_display(&ctx)
            )?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct UiCmd {
    #[command(flatten)]
    eval_options: EvalOptions,

    /// Path to the Ledger file.
    source: std::path::PathBuf,
}

impl UiCmd {
    fn validate(&self) -> Result<(), InvalidFlagError> {
        self.eval_options.validate()?;
        Ok(())
    }

    pub fn run(self) -> anyhow::Result<()> {
        // All report data is built inside `run_ui`: its session loop resets
        // the arena on reload (`r` / `F5`), which requires that nothing out
        // here borrows it.
        let reload = ui::report::ReloadContext::new(
            load::new_loader(self.source.clone()),
            load::new_loader(self.source.clone()),
            self.eval_options.to_process_options(),
            self.eval_options.to_conversion_spec(),
            self.eval_options.to_date_range()?,
        );
        let mut arena = Bump::new();
        ui::report::run_ui(&mut arena, self.source.display().to_string(), &reload)
            .context("failed to run TUI")
    }
}

/// `--sort` flag for `register`. `Original` keeps the file order; `Date`
/// sorts ascending by transaction date (stable, so ties preserve file order).
#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
pub enum SortKey {
    /// Preserve the order of appearance in the source file (current default).
    #[default]
    #[value(alias = "o")]
    Original,
    /// Stable sort by transaction date, ascending.
    #[value(alias = "d")]
    Date,
}

impl From<SortKey> for query::Sort {
    fn from(value: SortKey) -> Self {
        match value {
            SortKey::Original => query::Sort::Original,
            SortKey::Date => query::Sort::Date,
        }
    }
}

#[derive(Args, Debug)]
pub struct RegisterCmd {
    #[command(flatten)]
    eval_options: EvalOptions,

    /// Sort order for the register rows.
    #[arg(long, value_enum, default_value_t)]
    sort: SortKey,

    /// Path to the Ledger file.
    source: std::path::PathBuf,

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

impl RegisterCmd {
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            load::new_loader(self.source),
            &self.eval_options.to_process_options(),
        )?;
        let query = query::RegisterQuery {
            account: self
                .eval_options
                .create_account_filter(&ctx, self.account.as_slice())
                .context("failed to create regex for account filter")?,
            date_range: self.eval_options.to_date_range()?,
            conversion: self.eval_options.to_conversion(&ctx)?,
            sort: self.sort.into(),
        };
        let mut entries = ledger.register_entries(&ctx, &query)?;
        while let Some(entry) = entries.next()? {
            writeln!(
                w,
                "{} {} {} {} {}",
                entry.date,
                entry.payee,
                entry.account.as_str(),
                entry.amount.as_inline_display(&ctx),
                entry.total.as_inline_display(&ctx),
            )?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct EvalOptions {
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

/// Mode of the account filter.
#[derive(Debug, Default, Clone, Copy, clap::ValueEnum)]
enum AccountFilterMode {
    /// Use given account matcher as an unanchored regex.
    /// For example, `Bank` matches any accounts with `.*Bank.*`.
    #[default]
    Regex,
    /// Use given account matcher as the exact account name.
    /// For example, `Foo` matches only `Foo`.
    Exact,
}

impl EvalOptions {
    fn validate(&self) -> Result<(), InvalidFlagError> {
        if self.current && self.end.is_some() {
            return Err(InvalidFlagError(
                "--current and --end cannot be set simultaneously".to_string(),
            ));
        }
        Ok(())
    }

    fn to_process_options(&self) -> report::ProcessOptions {
        report::ProcessOptions {
            price_db_path: self.price_db.clone(),
        }
    }

    fn to_date_range(&self) -> anyhow::Result<query::DateRange> {
        let end = if self.current {
            let tomorrow = self.today.succ_opt().ok_or_else(|| {
                anyhow::anyhow!("cannot compute one day after today {}", self.today)
            })?;
            Some(tomorrow)
        } else {
            self.end
        };
        Ok(query::DateRange {
            start: self.start,
            end,
        })
    }

    fn conversion_strategy(&self) -> query::ConversionStrategy {
        if self.historical {
            query::ConversionStrategy::Historical
        } else {
            query::ConversionStrategy::UpToDate { today: self.today }
        }
    }

    fn to_conversion<'ctx>(
        &self,
        ctx: &report::ReportContext<'ctx>,
    ) -> Result<Option<query::Conversion<'ctx>>, query::QueryError> {
        let ex = match &self.exchange {
            None => return Ok(None),
            Some(ex) => ex,
        };
        let target = ctx
            .commodity(ex)
            .ok_or(query::QueryError::CommodityNotFound(
                report::OwnedCommodity::from_string(ex.clone()),
            ))?;
        Ok(Some(query::Conversion {
            strategy: self.conversion_strategy(),
            target,
        }))
    }

    /// Owned counterpart of [`Self::to_conversion`] for the UI session
    /// loop, which re-resolves it against each session's fresh context.
    fn to_conversion_spec(&self) -> Option<ui::report::ConversionSpec> {
        self.exchange.as_ref().map(|ex| ui::report::ConversionSpec {
            commodity: ex.clone(),
            strategy: self.conversion_strategy(),
        })
    }

    fn create_account_filter<'ctx>(
        &self,
        ctx: &report::ReportContext<'ctx>,
        accounts: &[String],
    ) -> Result<query::AccountFilter<'ctx>, regex::Error> {
        if accounts.is_empty() {
            return Ok(query::AccountFilter::All);
        }
        match self.account_filter {
            AccountFilterMode::Regex => query::AccountFilter::from_regex_patterns(ctx, accounts),
            AccountFilterMode::Exact => {
                Ok(query::AccountFilter::from_exact_accounts(ctx, accounts))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_command() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    fn import_cmd(interactive: bool, ledger: bool, output: bool) -> ImportCmd {
        ImportCmd {
            config: PathBuf::from("config.yml"),
            interactive,
            ledger: ledger.then(|| PathBuf::from("root.ledger")),
            output: output.then(|| PathBuf::from("out.ledger")),
            source: PathBuf::from("source.csv"),
        }
    }

    #[test]
    fn import_validate_accepts_batch_mode() {
        assert!(import_cmd(false, false, false).validate().is_ok());
    }

    #[test]
    fn import_validate_accepts_interactive_with_all_flags() {
        assert!(import_cmd(true, true, true).validate().is_ok());
    }

    #[test]
    fn import_validate_rejects_interactive_without_ledger_or_output() {
        assert!(import_cmd(true, false, true).validate().is_err());
        assert!(import_cmd(true, true, false).validate().is_err());
        assert!(import_cmd(true, false, false).validate().is_err());
    }
}
