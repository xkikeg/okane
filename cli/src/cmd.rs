use crate::format;
use crate::import::{self, Format, ImportError};
use okane_core::repl::display::DisplayContext;
use okane_core::{eval, load, repl};

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use bumpalo::Bump;
use clap::{Args, Subcommand};
use encoding_rs_io::DecodeReaderBytesBuilder;

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
    #[error("failed to evaluate the postings")]
    BalanceError(#[from] eval::error::BalanceError),
    // TODO: Move this into other module.
    #[error("register failed to find accounts {0}")]
    RegisterError(String),
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
    pub fn run(self) -> Result<(), Error> {
        match self {
            Command::Import(cmd) => cmd.run(&mut std::io::stdout().lock()),
            Command::Format(cmd) => cmd.run(&mut std::io::stdout().lock()),
            Command::Accounts(cmd) => cmd.run(&mut std::io::stdout().lock()),
            Command::Balance(cmd) => cmd.run(&mut std::io::stdout().lock()),
            Command::Register(cmd) => cmd.run(&mut std::io::stdout().lock()),
            Command::Primitive(cmd) => cmd.run(),
        }
    }
}

#[derive(Args, Debug)]
pub struct ImportCmd {
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,
    pub source: PathBuf,
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
        let ctx = repl::display::DisplayContext {
            precisions: config_entry
                .format
                .commodity
                .iter()
                .map(|(k, v)| (k.clone(), v.precision))
                .collect(),
        };
        for xact in xacts {
            let xact: repl::Transaction = xact.into();
            writeln!(w, "{}", ctx.as_display(&xact))?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct FormatCmd {
    pub source: PathBuf,
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
    fn run(self) -> Result<(), Error> {
        self.command.run()
    }
}

#[derive(Subcommand, Debug)]
enum PrimitiveCmd {
    /// Format the given one ledger file, to stdout.
    Format(FormatCmd),
    /// Read the given one ledger file, recursively resolves include directives and print to stdout.
    Flatten(FlattenCmd),
}

impl PrimitiveCmd {
    fn run(self) -> Result<(), Error> {
        match self {
            PrimitiveCmd::Format(cmd) => cmd.run(&mut std::io::stdout().lock()),
            PrimitiveCmd::Flatten(cmd) => cmd.run(&mut std::io::stdout().lock()),
        }
    }
}

#[derive(Args, Debug)]
struct FlattenCmd {
    pub source: PathBuf,
}

impl FlattenCmd {
    pub fn run<W>(&self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let entries = load::load_repl(&self.source)?;
        // TODO: Pick DisplayContext from load results.
        let ctx = DisplayContext::default();
        for entry in entries.iter() {
            writeln!(w, "{}", ctx.as_display(entry))?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AccountsCmd {
    pub source: PathBuf,
}

impl AccountsCmd {
    pub fn run<W>(&self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let entries = load::load_repl(&self.source)?;
        let arena = Bump::new();
        let mut ctx = eval::context::EvalContext::new(&arena);
        let accounts = eval::accounts(&mut ctx, &entries);
        for acc in accounts.iter() {
            writeln!(w, "{}", acc.as_str())?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct BalanceCmd {
    pub source: PathBuf,
}

impl BalanceCmd {
    pub fn run<W>(&self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let entries = eval::load(&self.source)?;
        let arena = Bump::new();
        let mut ctx = eval::context::EvalContext::new(&arena);
        let mut balance = eval::amounts::Balance::default();
        let txns: Result<Vec<eval::load::Transaction>, eval::error::BalanceError> = entries
            .iter()
            .filter_map(|x| {
                if let repl::LedgerEntry::Txn(txn) = x {
                    Some(txn)
                } else {
                    None
                }
            })
            .map(|txn| eval::load::balanced_txn(&mut ctx, &mut balance, txn))
            .collect();
        let _ = txns?;

        let mut accounts: Vec<eval::types::Account> = balance.accounts.keys().copied().collect();
        accounts.sort_by_key(|x| x.as_str());
        for account in &accounts {
            let amount = balance.accounts.get(account).unwrap();
            writeln!(w, "{}: {}", account.as_str(), amount.as_inline())?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct RegisterCmd {
    pub source: PathBuf,
    pub account: Option<String>,
}

impl RegisterCmd {
    pub fn run<W>(&self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let entries = eval::load(&self.source)?;
        let arena = Bump::new();
        let mut ctx = eval::context::EvalContext::new(&arena);
        let mut balance = eval::amounts::Balance::default();
        let txns: Result<Vec<eval::load::Transaction>, eval::error::BalanceError> = entries
            .iter()
            .filter_map(|x| {
                if let repl::LedgerEntry::Txn(txn) = x {
                    Some(txn)
                } else {
                    None
                }
            })
            .map(|txn| eval::load::balanced_txn(&mut ctx, &mut balance, txn))
            .collect();
        let txns = txns?;
        let account = self
            .account
            .as_ref()
            .map(|account| {
                ctx.accounts
                    .get(account)
                    .ok_or_else(|| Error::RegisterError(account.clone()))
            })
            .transpose()?;
        let mut balance = eval::amounts::Balance::default();
        for txn in &txns {
            if let Some(account) = &account {
                if let Some(p) = txn.postings.iter().find(|p| p.account == *account) {
                    let b = balance.accounts.entry(*account).or_default();
                    *b += p.amount.clone();
                    writeln!(
                        w,
                        "{} {} {}",
                        account.as_str(),
                        p.amount.as_inline(),
                        b.as_inline()
                    )?;
                }
                continue;
            }
            writeln!(w, "{:#?}", txn)?;
        }
        Ok(())
    }
}
