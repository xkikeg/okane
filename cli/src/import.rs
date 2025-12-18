// Define converter interface here.

mod amount;
pub mod config;
pub mod csv;
mod error;
pub mod extract;
pub mod iso_camt053;
pub mod single_entry;
pub(crate) mod template;
pub mod viseca;

pub use error::ImportError;

use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use bumpalo::Bump;
use encoding_rs_io::DecodeReaderBytesBuilder;

use okane_core::report;
use okane_core::syntax::{self, display::DisplayContext};

/// Operates file import into Ledger format.
pub struct Importer {
    config_set: config::ConfigSet,
}

impl Importer {
    pub fn new(config_path: &Path) -> Result<Self, ImportError> {
        let config_file = File::open(config_path)?;
        let config_set = config::load_from_yaml(config_file)?;
        Ok(Self { config_set })
    }

    pub fn import<W: std::io::Write>(
        &self,
        source_path: &Path,
        w: &mut W,
    ) -> Result<(), ImportError> {
        let config_entry = self.config_set.select(source_path)?.ok_or_else(|| {
            ImportError::Other(format!(
                "config matching {} not found",
                source_path.display()
            ))
        })?;
        log::debug!("config: {:?}", config_entry);
        // TODO: Allow injecting source format with config.
        // https://github.com/xkikeg/okane/issues/319
        let format = match source_path.extension().and_then(OsStr::to_str) {
            Some("csv") => Ok(Format::Csv("")),
            Some("tsv") => Ok(Format::Csv("\t")),
            Some("xml") => Ok(Format::IsoCamt053),
            Some("txt") => Ok(Format::Viseca),
            _ => Err(ImportError::UnknownFormat),
        }?;
        let file = File::open(source_path)?;
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(config_entry.encoding.as_encoding()))
            .build(file);
        let txns: Vec<single_entry::Txn> = match format {
            Format::Csv(delimiter) => csv::import(decoded, delimiter, &config_entry),
            Format::IsoCamt053 => iso_camt053::import(decoded, &config_entry),
            Format::Viseca => viseca::import(decoded, &config_entry),
        }?;
        let ctx = DisplayContext {
            commodity: config_entry.output.commodity.into(),
        };
        let opts = single_entry::Options {
            operator: config_entry.operator.clone(),
            commodity_rename: config_entry.commodity.rename.clone(),
            commodity_format: ctx.commodity.clone(),
        };
        let arena = Bump::new();
        let mut rctx = report::ReportContext::new(&arena);
        for txn in &txns {
            let txn: syntax::plain::Transaction =
                txn.to_double_entry(&config_entry.account, &opts, &mut rctx)?;
            writeln!(w, "{}", ctx.as_display(&txn))?;
        }
        Ok(())
    }
}

/// Format of the supported importer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Format {
    Csv(&'static str),
    IsoCamt053,
    Viseca,
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;

    fn simple_config() -> config::Config {
        config::Config {
            path: "matched/".to_string(),
            encoding: config::Encoding(encoding_rs::UTF_8),
            account: "Assets:Bank:Okane".to_string(),
            account_type: config::AccountType::Asset,
            operator: None,
            commodity: config::AccountCommoditySpec::default(),
            format: config::FormatSpec::default(),
            output: config::OutputSpec::default(),
            rewrite: Vec::new(),
        }
    }

    #[test]
    fn import_fails_when_no_config_matches() {
        let importer = Importer {
            config_set: config::ConfigSet::from_configs([simple_config()]),
        };
        let mut buf: Vec<u8> = Vec::new();

        assert_matches!(
            importer.import(Path::new("unrelated.csv"), &mut buf),
            Err(ImportError::Other(msg)) => {
                assert!(msg.contains("config matching unrelated.csv not found"), "msg {msg:?} does not match");
            }
        );
    }

    #[test]
    fn import_fails_when_no_format_specified() {
        let importer = Importer {
            config_set: config::ConfigSet::from_configs(vec![simple_config()]),
        };
        let mut buf: Vec<u8> = Vec::new();

        assert_matches!(
            importer.import(Path::new("matched/unknown_format"), &mut buf),
            Err(ImportError::UnknownFormat)
        );
    }
}
