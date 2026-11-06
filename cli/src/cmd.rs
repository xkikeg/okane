//! The commands okane runs.
//!
//! The command line is parsed into [`args`]' structs first, which `build()`
//! the commands here. A command holds only valid states (e.g. an interactive
//! import always has its `--ledger` and `--output`), so running one never
//! re-checks its flags.

mod args;

pub use args::Cli;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use bumpalo::Bump;
use chrono::NaiveDate;

use lender::FallibleLender;
use okane_core::report::query;
use okane_core::syntax::Separation;
use okane_core::syntax::display::DisplayContext;
use okane_core::syntax::plain::LedgerEntry;
use okane_core::{load, report};

use crate::format;
use crate::import;
use crate::ui;

#[derive(thiserror::Error, Debug)]
#[error("invalid flag: {0}")]
pub struct InvalidFlagError(String);

#[derive(Debug)]
pub enum Command {
    Import(ImportCmd),
    Format(FormatCmd),
    Accounts(AccountsCmd),
    Tags(TagsCmd),
    Balance(BalanceCmd),
    Register(RegisterCmd),
    Ui(UiCmd),
    Primitive(PrimitiveCmd),
}

impl Command {
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Command::Import(cmd) => cmd.run(w),
            Command::Format(cmd) => cmd.run(w),
            Command::Accounts(cmd) => cmd.run(w),
            Command::Tags(cmd) => cmd.run(w),
            Command::Balance(cmd) => cmd.run(w),
            Command::Register(cmd) => cmd.run(w),
            Command::Ui(cmd) => cmd.run(),
            Command::Primitive(cmd) => cmd.run(w),
        }
    }
}

#[derive(Debug)]
pub struct ImportCmd {
    /// YAML file with the import rules.
    rules: PathBuf,
    mode: ImportMode,
    /// File to import.
    source: PathBuf,
}

/// Whether `import` prints the transactions, or lets the user review them.
#[derive(Debug)]
enum ImportMode {
    /// Write the imported transactions to stdout.
    Batch,
    /// Review the imported transactions in the TUI, then append them to
    /// `output`. `ledger` is the file whose accounts feed the autocomplete.
    Interactive { ledger: PathBuf, output: PathBuf },
}

impl ImportCmd {
    pub fn run<W>(&self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        match &self.mode {
            ImportMode::Batch => {
                let importer = import::Importer::new(&self.rules)?;
                importer.import_and_write(&self.source, w)?;
                Ok(())
            }
            ImportMode::Interactive { ledger, output } => self.run_interactive(ledger, output),
        }
    }

    fn run_interactive(&self, ledger_path: &Path, output_path: &Path) -> anyhow::Result<()> {
        let importer = import::Importer::new(&self.rules)?;
        let (header, mut txns) = importer.import(&self.source)?;
        if txns.is_empty() {
            eprintln!("no transactions found in {}", self.source.display());
            return Ok(());
        }
        let accounts: Vec<String> = {
            let arena = Bump::new();
            let mut ctx = report::ReportContext::new(&arena);
            report::accounts(&mut ctx, load::new_loader(ledger_path.to_owned()))
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

#[derive(Debug)]
pub struct FormatCmd {
    source: PathBuf,
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

#[derive(Debug)]
pub enum PrimitiveCmd {
    Format(FormatCmd),
    Flatten(FlattenCmd),
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

#[derive(Debug)]
pub struct FlattenCmd {
    source: PathBuf,
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
        let mut prev_path: Option<PathBuf> = None;
        load::new_loader(self.source).load(
            |path, _ctx, entry: &LedgerEntry| -> Result<(), FlattenError> {
                let file_changed = prev_path.as_deref().is_some_and(|prev| prev != path);
                if file_changed {
                    prev_path = Some(path.to_owned());
                    // The first entry of every file is `Separation::Immediate`, so without this
                    // the last entry of one file would run into the first entry of the next.
                    if entry.separation == Separation::Immediate {
                        writeln!(w)?;
                    }
                }
                write!(w, "{}", ctx.as_display(entry))?;
                Ok(())
            },
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EvalCmd {
    /// Date of the price.
    date: NaiveDate,
    eval_options: EvalOptions,
    source: PathBuf,
    /// Terms of the expression, joined with spaces.
    expression: Vec<String>,
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

#[derive(Debug)]
pub struct AccountsCmd {
    source: PathBuf,
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

#[derive(Debug)]
pub struct TagsCmd {
    source: PathBuf,
    /// Print each tag as `key: value`, instead of the key alone.
    values: bool,
}

impl TagsCmd {
    pub fn run<W>(self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let query = if self.values {
            report::TagQuery::WithValues
        } else {
            report::TagQuery::KeysOnly
        };
        let tags = report::tags(load::new_loader(self.source), query)?;
        for tag in tags {
            match tag.value {
                None => writeln!(w, "{}", tag.key),
                Some(report::TagValue::Expr(expr)) => writeln!(w, "{}:: {}", tag.key, expr),
                Some(report::TagValue::Text(text)) => writeln!(w, "{}: {}", tag.key, text),
            }?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BalanceCmd {
    eval_options: EvalOptions,
    source: PathBuf,
    /// Account patterns to report; empty for every account.
    account: Vec<String>,
}

impl BalanceCmd {
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
            date_range: self.eval_options.date_range,
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

#[derive(Debug)]
pub struct UiCmd {
    eval_options: EvalOptions,
    source: PathBuf,
}

impl UiCmd {
    pub fn run(self) -> anyhow::Result<()> {
        // All report data is built inside `run_ui`: its session loop resets
        // the arena on reload (`r` / `F5`), which requires that nothing out
        // here borrows it.
        let config = ui::report::SessionConfig::new(
            load::new_loader(self.source.clone()),
            self.eval_options.to_query_options(),
        );
        let mut arena = Bump::new();
        ui::report::run_ui(&mut arena, self.source.display().to_string(), &config)
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

#[derive(Debug)]
pub struct RegisterCmd {
    eval_options: EvalOptions,
    sort: SortKey,
    source: PathBuf,
    /// Account patterns to report; empty for every account.
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
            date_range: self.eval_options.date_range,
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

/// How the reports are evaluated, shared by `balance`, `register`, `ui` and
/// `primitive eval`.
#[derive(Debug)]
struct EvalOptions {
    price_db: Option<PathBuf>,
    /// Commodity every amount is converted into, if any.
    exchange: Option<String>,
    /// Convert at the rate of each transaction's date, instead of `today`'s.
    historical: bool,
    today: NaiveDate,
    /// Transactions to report, with `--current` already turned into its end.
    date_range: query::DateRange,
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
    fn to_process_options(&self) -> report::ProcessOptions {
        report::ProcessOptions {
            price_db_path: self.price_db.clone(),
        }
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

    /// Owned counterpart of [`Self::to_conversion`] for the UI, which
    /// re-resolves the query against every session's fresh context — and lets
    /// the user restate it from inside the TUI.
    fn to_query_options(&self) -> ui::report::QueryOptions {
        ui::report::QueryOptions {
            price_db: self.price_db.clone(),
            exchange: self.exchange.clone(),
            historical: self.historical,
            today: self.today,
            start: self.date_range.start,
            end: self.date_range.end,
        }
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
