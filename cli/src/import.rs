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

pub use error::{ImportError, ImportErrorKind};

use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bumpalo::Bump;
use encoding_rs_io::DecodeReaderBytesBuilder;

use okane_core::report;
use okane_core::syntax::{self, display::DisplayContext};

use error::IntoImportError;

/// Operates file import into Ledger format.
#[derive(Debug)]
pub struct Importer {
    config_set: config::ConfigSet,
}

impl Importer {
    pub fn new(config_path: &Path) -> Result<Self, ImportError> {
        let config_file = File::open(config_path)
            .into_import_err(ImportErrorKind::ConfigFileReadFailed, || {
                format!("failed to read config file {}", config_path.display())
            })?;
        let config_set = config::load_from_yaml(config_file)?;
        Ok(Self { config_set })
    }

    /// Parses the source file into transactions.
    pub fn import(
        &self,
        source_path: &Path,
    ) -> Result<(ImportHeader, Vec<single_entry::Txn>), ImportError> {
        let config = self
            .config_set
            .select(source_path)?
            .into_import_err(ImportErrorKind::ConfigNotFound, || {
                format!("config for {} is not found", source_path.display())
            })?;
        log::debug!("config: {:?}", config);
        let format = Format::try_new(config.format.file_type, source_path)?;
        let file = File::open(source_path)
            .into_import_err(ImportErrorKind::SourceFileReadFailed, || {
                format!("failed to read source file {}", source_path.display())
            })?;
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(config.encoding.as_encoding()))
            .build(file);
        let txns: Vec<single_entry::Txn> = match format {
            Format::Csv(delimiter) => csv::import(decoded, delimiter, &config),
            Format::IsoCamt053 => iso_camt053::import(decoded, &config),
            Format::Viseca => viseca::import(decoded, &config),
        }?;
        let display_context = DisplayContext {
            commodity: config.output.commodity.into(),
        };
        let options = single_entry::Options {
            operator: config.operator.clone(),
            commodity_rename: config.commodity.rename.clone(),
            commodity_format: display_context.commodity.clone(),
        };
        Ok((
            ImportHeader {
                src_account: config.account,
                options,
                display_context,
            },
            txns,
        ))
    }

    pub fn import_and_write<W: Write>(
        &self,
        source_path: &Path,
        w: &mut W,
    ) -> Result<(), ImportError> {
        let (header, txns) = self.import(source_path)?;
        let arena = Bump::new();
        let mut rctx = report::ReportContext::new(&arena);
        for (i, txn) in txns.iter().enumerate() {
            let txn: syntax::plain::Transaction =
                txn.to_double_entry(&header.src_account, &header.options, &mut rctx)?;
            writeln!(w, "{}", header.display_context.as_display(&txn))
                .into_import_err(ImportErrorKind::OutputFailed, || {
                    format!("output failed at transaction {}", i + 1)
                })?;
        }
        Ok(())
    }
}

/// Render metadata from one source file import: everything needed to convert
/// a [`single_entry::Txn`] into a formatted Ledger entry. Immutable once
/// constructed; the interactive review only mutates the accompanying
/// `Vec<single_entry::Txn>`.
#[derive(Debug)]
pub struct ImportHeader {
    /// Ledger account associated with the source file (`config.account`).
    pub src_account: String,
    pub options: single_entry::Options,
    pub display_context: DisplayContext,
}

impl ImportHeader {
    pub fn render_transaction(&self, txn: &single_entry::Txn) -> Result<String, ImportError> {
        let arena = Bump::new();
        let mut rctx = report::ReportContext::new(&arena);
        let txn = txn.to_double_entry(&self.src_account, &self.options, &mut rctx)?;
        Ok(self.display_context.as_display(&txn).to_string())
    }
}

/// Format of the supported importer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Format {
    Csv(&'static str),
    IsoCamt053,
    Viseca,
}

impl Format {
    fn try_new(config_value: Option<config::FileType>, path: &Path) -> Result<Self, ImportError> {
        config_value
            .map(Self::from_config)
            .or_else(|| Self::from_path(path))
            .into_import_err(ImportErrorKind::UnknownSourceFileFormat, || {
                format!("config.format.file_type must be set for {}", path.display())
            })
    }

    fn from_config(v: config::FileType) -> Self {
        match v {
            config::FileType::Csv => Self::Csv(""),
            config::FileType::Tsv => Self::Csv("\t"),
            config::FileType::IsoCamt053 => Self::IsoCamt053,
            config::FileType::Viseca => Self::Viseca,
        }
    }

    fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(OsStr::to_str) {
            Some("csv") => Some(Format::Csv("")),
            Some("tsv") => Some(Format::Csv("\t")),
            _ => None,
        }
    }
}

/// Appends the rendered transactions to `path` in one write, creating the
/// file when missing. A blank line separates the existing content from the
/// new entries and each entry, matching the batch `import` output.
pub fn append_transactions(path: &std::path::Path, rendered: &[String]) -> Result<(), ImportError> {
    use std::io::Write as _;

    let existing = match std::fs::read(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(err) => {
            return Err(ImportError::with_source(
                ImportErrorKind::OutputFailed,
                format!("failed to read {}", path.display()),
                err,
            ));
        }
    };
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .into_import_err(ImportErrorKind::OutputFailed, || {
            format!("failed to open output file {}", path.display())
        })?;
    let out = std::io::BufWriter::new(file);
    // we assume that output is always UTF-8.
    (|mut out: std::io::BufWriter<_>| {
        {
            out.write_all(entry_separator(&existing).as_bytes())?;
            for txn in rendered {
                // Each rendered entry already ends with '\n'; one more gives the
                // blank-line separation `import` emits between entries.
                out.write_all(txn.as_bytes())?;
                out.write_all(b"\n")?;
            }
            out.flush()
        }
    })(out)
    .into_import_err(ImportErrorKind::OutputFailed, || {
        format!("failed to write the result to {}", path.display())
    })?;
    Ok(())
}

/// Separator ensuring a blank line between existing content and appended
/// entries.
fn entry_separator(existing: &[u8]) -> &'static str {
    if existing.is_empty() || existing.ends_with(b"\n\n") {
        ""
    } else if existing.ends_with(b"\n") {
        "\n"
    } else {
        "\n\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;

    mod format {
        use super::*;

        use pretty_assertions::assert_eq;

        #[test]
        fn try_new_prefers_config_specified_value() {
            assert_eq!(
                Format::Viseca,
                Format::try_new(Some(config::FileType::Viseca), Path::new("test.csv")).unwrap()
            );
            assert_eq!(
                Format::Csv("\t"),
                Format::try_new(Some(config::FileType::Tsv), Path::new("test.csv")).unwrap()
            );
        }

        #[test]
        fn try_new_fallback_to_guess() {
            assert_eq!(
                Format::Csv(""),
                Format::try_new(None, Path::new("test.csv")).unwrap()
            );
            assert_eq!(
                Format::Csv("\t"),
                Format::try_new(None, Path::new("test.tsv")).unwrap()
            );
        }

        #[test]
        fn try_new_fails_on_unknown_suffix() {
            assert_matches!(
                Format::try_new(None, Path::new("test.xml")),
                Err(err) => assert_eq!(ImportErrorKind::UnknownSourceFileFormat, err.error_kind())
            );
        }
    }

    #[test]
    fn importer_creation_fails_on_config_read_failure() {
        let got_err = Importer::new(Path::new("does/not/exist.yml")).unwrap_err();

        assert_eq!(ImportErrorKind::ConfigFileReadFailed, got_err.error_kind());
    }

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

        let got_err = importer
            .import_and_write(Path::new("unrelated.csv"), &mut buf)
            .unwrap_err();

        assert_eq!(ImportErrorKind::ConfigNotFound, got_err.error_kind());
        assert_eq!("config for unrelated.csv is not found", got_err.message());
    }

    #[test]
    fn import_fails_when_no_format_specified() {
        let importer = Importer {
            config_set: config::ConfigSet::from_configs(vec![simple_config()]),
        };
        let mut buf: Vec<u8> = Vec::new();

        let got_err = importer
            .import_and_write(Path::new("matched/unknown_format"), &mut buf)
            .unwrap_err();

        assert_eq!(
            ImportErrorKind::UnknownSourceFileFormat,
            got_err.error_kind()
        );
    }

    #[test]
    fn import_fails_when_file_not_found() {
        let importer = Importer {
            config_set: config::ConfigSet::from_configs(vec![simple_config()]),
        };
        let mut buf: Vec<u8> = Vec::new();

        let got_err = importer
            .import_and_write(Path::new("matched/not_existing.csv"), &mut buf)
            .unwrap_err();

        assert_eq!(ImportErrorKind::SourceFileReadFailed, got_err.error_kind());
    }

    #[test]
    fn load_classifies_transactions_for_review() {
        let testdata_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/import/");
        let importer = Importer::new(&testdata_dir.join("test_config.yml")).unwrap();

        let (_header, txns) = importer
            .import(&testdata_dir.join("index_amount.csv"))
            .unwrap();

        use single_entry::ReviewKind;
        let kinds: Vec<ReviewKind> = txns.iter().map(|t| t.review_kind()).collect();
        assert_eq!(
            vec![
                ReviewKind::Pending,
                ReviewKind::Auto,
                ReviewKind::Unknown,
                ReviewKind::Unknown
            ],
            kinds
        );
    }

    #[test]
    fn render_transaction_matches_batch_output() {
        let testdata_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/import/");
        let importer = Importer::new(&testdata_dir.join("test_config.yml")).unwrap();
        let target = testdata_dir.join("index_amount.csv");

        let mut batch: Vec<u8> = Vec::new();
        importer.import_and_write(&target, &mut batch).unwrap();
        let batch = String::from_utf8(batch).unwrap();

        let (header, txns) = importer.import(&target).unwrap();
        let rendered: String = txns
            .iter()
            .map(|txn| header.render_transaction(txn).unwrap() + "\n")
            .collect();

        assert_eq!(batch, rendered);
    }

    #[test]
    fn import_fails_when_failed_to_write_output() {
        let testdata_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/import/");
        // buf is just 1 byte buffer, fails to write long.
        let mut buf = [0u8; 1];
        let importer = Importer::new(&testdata_dir.join("test_config.yml")).unwrap();
        let target = testdata_dir.join("index_amount.csv");

        let mut buf_slice = &mut buf[..];
        let got_err = importer
            .import_and_write(&target, &mut buf_slice)
            .unwrap_err();
        assert_eq!(ImportErrorKind::OutputFailed, got_err.error_kind());
        assert!(
            got_err.message().contains("at transaction 1"),
            "original message: {}",
            got_err.message()
        );
    }

    #[test]
    fn entry_separator_cases() {
        assert_eq!(entry_separator(b""), "");
        assert_eq!(entry_separator(b"txn\n\n"), "");
        assert_eq!(entry_separator(b"txn\n"), "\n");
        assert_eq!(entry_separator(b"txn"), "\n\n");
    }

    fn rendered() -> Vec<String> {
        vec!["2024/01/02 * A\n    X    1 USD\n".to_string()]
    }

    #[test]
    fn append_transactions_creates_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.ledger");

        append_transactions(&path, &rendered()).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "2024/01/02 * A\n    X    1 USD\n\n"
        );
    }

    #[test]
    fn append_transactions_separates_with_blank_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.ledger");
        std::fs::write(&path, "2023/12/31 * Old\n    Y    2 USD\n").unwrap();

        append_transactions(&path, &rendered()).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "2023/12/31 * Old\n    Y    2 USD\n\n2024/01/02 * A\n    X    1 USD\n\n"
        );
    }

    #[test]
    fn append_transactions_completes_missing_final_newline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.ledger");
        std::fs::write(&path, "; note without newline").unwrap();

        append_transactions(&path, &rendered()).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "; note without newline\n\n2024/01/02 * A\n    X    1 USD\n\n"
        );
    }

    #[test]
    fn append_transactions_keeps_existing_blank_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.ledger");
        std::fs::write(&path, "2023/12/31 * Old\n    Y    2 USD\n\n").unwrap();

        append_transactions(&path, &rendered()).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "2023/12/31 * Old\n    Y    2 USD\n\n2024/01/02 * A\n    X    1 USD\n\n"
        );
    }

    #[test]
    fn append_transactions_multiple_entries_blank_line_separated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.ledger");
        let txns = vec!["a\n".to_string(), "b\n".to_string()];

        append_transactions(&path, &txns).unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "a\n\nb\n\n");
    }
}
