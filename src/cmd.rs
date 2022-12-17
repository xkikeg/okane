use crate::format;
use crate::import::{self, Format, ImportError};
use crate::repl;

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;

use clap::Args;
use encoding_rs_io::DecodeReaderBytesBuilder;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to import")]
    Import(#[from] import::ImportError),
    #[error("failed to format")]
    Format(#[from] format::FormatError),
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
            let xact: repl::Transaction = xact.into();
            writeln!(w, "{}", ctx.wrap(&xact))?;
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
