use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use bumpalo::Bump;
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use encoding_rs_io::DecodeReaderBytesBuilder;

use okane_core::report::query;
use okane_core::syntax::display::DisplayContext;
use okane_core::syntax::plain::LedgerEntry;
use okane_core::{load, report, syntax};

use crate::format;
use crate::import::{self, Format, ImportError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to import")]
    Import(#[from] import::ImportError),
    #[error("failed to format")]
    Format(#[from] format::FormatError),
    #[error("failed to load")]
    Load(#[from] load::LoadError),
    #[error("failed to report")]
    Report(#[from] report::ReportError),
    #[error("failed to query")]
    Query(#[from] query::QueryError),
}

#[derive(clap::Parser, Debug)]
#[clap(about, version, author)]
#[command(infer_subcommands = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
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
    /// Primitive is a set of commands which are primitive and suitable for debugging.
    Primitive(Primitives),
}

impl Command {
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        match self {
            Command::Import(cmd) => cmd.run(w),
            Command::Format(cmd) => cmd.run(w),
            Command::Accounts(cmd) => cmd.run(w),
            Command::Balance(cmd) => cmd.run(w),
            Command::Register(cmd) => cmd.run(w),
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
    pub fn run<W>(&self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let config_file = File::open(&self.config)?;
        let config_set = import::config::load_from_yaml(config_file)?;
        let config_entry = config_set.select(&self.source)?.ok_or_else(|| {
            ImportError::Other(format!(
                "config matching {} not found",
                self.source.display()
            ))
        })?;
        log::debug!("config: {:?}", config_entry);
        let file = File::open(&self.source)?;
        // Use dedicated flags or config systems instead.
        let format = match self.source.extension().and_then(OsStr::to_str) {
            Some("csv") => Ok(Format::Csv),
            Some("xml") => Ok(Format::IsoCamt053),
            Some("txt") => Ok(Format::Viseca),
            _ => Err(ImportError::UnknownFormat),
        }?;
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(config_entry.encoding.as_encoding()))
            .build(file);
        let xacts = import::import(decoded, format, &config_entry)?;
        let ctx = syntax::display::DisplayContext {
            precisions: config_entry
                .format
                .commodity
                .iter()
                .map(|(k, v)| (k.clone(), v.precision))
                .collect(),
        };
        let opts = import::single_entry::Options {
            commodity_rename: config_entry.commodity.rename.clone(),
        };
        for xact in xacts {
            let xact: syntax::plain::Transaction =
                xact.to_double_entry(&config_entry.account, &opts)?;
            writeln!(w, "{}", ctx.as_display(&xact))?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct FormatCmd {
    pub source: std::path::PathBuf,
}

impl FormatCmd {
    pub fn run<W>(&self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let mut r = BufReader::new(File::open(&self.source)?);
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
    fn run<W>(self, w: &mut W) -> Result<(), Error>
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
    fn run<W>(self, w: &mut W) -> Result<(), Error>
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
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        // TODO: Pick DisplayContext from load results.
        let ctx = DisplayContext::default();
        load::new_loader(self.source).load(
            |_path, _ctx, entry: &LedgerEntry| -> Result<(), Error> {
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
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
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
        writeln!(w, "{}", result.as_inline_display())?;
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AccountsCmd {
    pub source: std::path::PathBuf,
}

impl AccountsCmd {
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
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
}

impl BalanceCmd {
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
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
        let query = query::BalanceQuery {
            conversion: self.eval_options.to_conversion(&ctx)?,
            date_range: self.eval_options.to_date_range(),
        };
        for (account, amount) in ledger.balance(&ctx, &query)?.into_owned().into_vec() {
            writeln!(w, "{}: {}", account.as_str(), amount.as_inline_display())?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct RegisterCmd {
    #[command(flatten)]
    eval_options: EvalOptions,

    /// Path to the Ledger file.
    source: std::path::PathBuf,

    /// [Optional] Account to track the register.
    account: Option<String>,
}

impl RegisterCmd {
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let ledger = report::process(
            &mut ctx,
            load::new_loader(self.source),
            &self.eval_options.to_process_options(),
        )?;
        let postings = ledger.postings(
            &ctx,
            &query::PostingQuery {
                account: self.account.clone(),
            },
        );
        let mut balance = report::Amount::default();
        for posting in postings {
            balance += posting.amount.clone();
            writeln!(
                w,
                "{} {} {}",
                posting.account.as_str(),
                posting.amount.as_inline_display(),
                balance.as_inline_display()
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

    #[arg(long, default_value_t = chrono::Local::now().date_naive())]
    now: NaiveDate,

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
}

impl EvalOptions {
    fn to_process_options(&self) -> report::ProcessOptions {
        report::ProcessOptions {
            price_db_path: self.price_db.clone(),
        }
    }

    fn to_date_range(&self) -> query::DateRange {
        query::DateRange {
            start: self.start,
            end: self.end,
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
        let strategy = if self.historical {
            query::ConversionStrategy::Historical
        } else {
            query::ConversionStrategy::UpToDate { now: self.now }
        };
        Ok(Some(query::Conversion { strategy, target }))
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
