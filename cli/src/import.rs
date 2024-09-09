// Define converter interface here.

pub mod amount;
pub mod config;
pub mod csv;
pub mod extract;
pub mod iso_camt053;
pub mod single_entry;
pub(crate) mod template;
pub mod viseca;

mod error;

pub use error::ImportError;

/// Format of the supported importer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Csv,
    IsoCamt053,
    Viseca,
}

pub fn import<R: std::io::Read>(
    r: R,
    fmt: Format,
    config: &config::ConfigEntry,
) -> Result<Vec<single_entry::Txn>, ImportError> {
    match fmt {
        Format::Csv => csv::CsvImporter {}.import(r, config),
        Format::IsoCamt053 => iso_camt053::IsoCamt053Importer {}.import(r, config),
        Format::Viseca => viseca::VisecaImporter {}.import(r, config),
    }
}

/// Trait the each format should implement, to be used in import() internally.
pub trait Importer {
    fn import<R: std::io::Read>(
        &self,
        r: R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError>;
}
