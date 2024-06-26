use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;

use bumpalo::Bump;
use clap::{Args, Subcommand};
use encoding_rs_io::DecodeReaderBytesBuilder;

use okane_core::repl::display::DisplayContext;
use okane_core::{load, repl, report};

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
        let ctx = repl::display::DisplayContext {
            precisions: config_entry
                .format
                .commodity
                .iter()
                .map(|(k, v)| (k.clone(), v.precision))
                .collect(),
        };
        for xact in xacts {
            let xact: repl::Transaction = (&xact).into();
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
    pub source: std::path::PathBuf,
}

impl FlattenCmd {
    pub fn run<W>(self, w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        // TODO: Pick DisplayContext from load results.
        let ctx = DisplayContext::default();
        load::Loader::new(self.source).load_repl(|_, entry| -> Result<(), Error> {
            writeln!(w, "{}", ctx.as_display(entry))?;
            Ok(())
        })?;
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
        let accounts = report::accounts(&mut ctx, load::Loader::new(self.source))?;
        for acc in accounts.iter() {
            writeln!(w, "{}", acc.as_str())?;
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct BalanceCmd {
    source: std::path::PathBuf,
}

impl BalanceCmd {
    pub fn run<W>(&self, _w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        todo!("not implemented");
    }
}

#[derive(Args, Debug)]
pub struct RegisterCmd {
    source: std::path::PathBuf,
}

impl RegisterCmd {
    pub fn run<W>(&self, _w: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        todo!("not implemented");
    }
}
