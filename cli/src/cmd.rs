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
    pub source: std::path::PathBuf,
}

impl ImportCmd {
    pub fn run<W>(&self, w: &mut W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let importer = import::Importer::new(&self.config)?;
        importer.import(&self.source, w)?;
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

    /// Show a flat list of accounts instead of the hierarchical tree.
    #[arg(long, default_value_t)]
    flat: bool,
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
        let balance = ledger.balance(&ctx, &query)?.into_owned();
        if self.flat {
            for (account, amount) in balance.into_vec() {
                writeln!(
                    w,
                    "{}: {}",
                    account.as_str(),
                    amount.as_inline_display(&ctx)
                )?;
            }
        } else {
            print_balance_tree(w, &ctx, &report::AccountTree::from_balance(&balance))?;
        }
        Ok(())
    }
}

/// Minimum width of the amount column in the tree-style balance report,
/// matching ledger-cli's default.
const BALANCE_AMOUNT_WIDTH: usize = 20;

/// Renders the balance as an indented tree in ledger-cli style:
/// rolled-up amounts on every level, single-child chains collapsed into
/// `Parent:Child`, and a grand total after a dashed rule.
fn print_balance_tree<'ctx, W>(
    w: &mut W,
    ctx: &'ctx report::ReportContext<'ctx>,
    tree: &'ctx report::AccountTree<'ctx>,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    struct Row {
        depth: usize,
        name: String,
        amount: String,
    }

    fn collect_rows<'ctx>(
        ctx: &'ctx report::ReportContext<'ctx>,
        node: report::AccountNode<'ctx, 'ctx>,
        depth: usize,
        rows: &mut Vec<Row>,
    ) {
        let mut name = node.segment().to_string();
        let mut node = node;
        // Collapse pure intermediate levels (no own postings, one child)
        // into a single `Parent:Child` row, as ledger-cli does.
        while node.own_amount().is_none() {
            let mut children = node.children();
            let (Some(child), None) = (children.next(), children.next()) else {
                break;
            };
            name.push(':');
            name.push_str(child.segment());
            node = child;
        }
        rows.push(Row {
            depth,
            name,
            amount: node.total().as_inline_display(ctx).to_string(),
        });
        for child in node.children() {
            collect_rows(ctx, child, depth + 1, rows);
        }
    }

    let mut rows = Vec::new();
    for root in tree.roots() {
        collect_rows(ctx, root, 0, &mut rows);
    }
    if rows.is_empty() {
        return Ok(());
    }
    let total = tree.total().as_inline_display(ctx).to_string();
    let width = rows
        .iter()
        .map(|row| row.amount.chars().count())
        .chain([total.chars().count(), BALANCE_AMOUNT_WIDTH])
        .max()
        .expect("rows is non-empty");
    for row in &rows {
        writeln!(
            w,
            "{:>width$}  {:indent$}{}",
            row.amount,
            "",
            row.name,
            indent = row.depth * 2
        )?;
    }
    writeln!(w, "{}", "-".repeat(width))?;
    writeln!(w, "{total:>width$}")?;
    Ok(())
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
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let mut ledger = report::process(
            &mut ctx,
            load::new_loader(self.source.clone()),
            &self.eval_options.to_process_options(),
        )?;
        let conversion = self.eval_options.to_conversion(&ctx)?;
        let date_range = self.eval_options.to_date_range()?;
        let balance_query = query::BalanceQuery {
            account: query::AccountFilter::All,
            conversion,
            date_range,
        };
        let balance = ledger.balance(&ctx, &balance_query)?.into_owned();
        let rows: Vec<ui::BalanceRow> = balance
            .into_vec()
            .into_iter()
            .map(|(account, amount)| ui::BalanceRow { account, amount })
            .collect();
        // Reused for every register drill-down during the session.
        let register_template = ui::RegisterQueryTemplate {
            conversion,
            date_range,
        };
        let app = ui::App::new(self.source.display().to_string(), rows, register_template);
        ui::run_ui(app, &mut ledger, &ctx).context("failed to run TUI")?;
        Ok(())
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
        let strategy = if self.historical {
            query::ConversionStrategy::Historical
        } else {
            query::ConversionStrategy::UpToDate { today: self.today }
        };
        Ok(Some(query::Conversion { strategy, target }))
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
}
