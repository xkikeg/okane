// Define converter interface here.

pub mod config;
pub mod csv;
pub mod extract;
pub mod iso_camt053;
pub mod single_entry;
pub mod viseca;

mod error;

pub use error::ImportError;

use crate::data::Transaction;

/// Format of the supported importer.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Format {
    Csv,
    IsoCamt053,
    Viseca,
}

pub fn import<R: std::io::Read>(
    r: R,
    fmt: Format,
    config: &config::ConfigEntry,
) -> Result<Vec<Transaction>, ImportError> {
    let txns = match fmt {
        Format::Csv => csv::CsvImporter {}.import(r, config),
        Format::IsoCamt053 => iso_camt053::IsoCamt053Importer {}.import(r, config),
        Format::Viseca => viseca::VisecaImporter {}.import(r, config),
    }?;
    let mut res = Vec::new();
    for txn in txns {
        let de = txn.to_double_entry(config.account.as_str())?;
        res.push(de);
    }
    Ok(res)
}

/// Trait the each format should implement, to be used in import() internally.
pub trait Importer {
    fn import<R: std::io::Read>(
        &self,
        r: R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError>;
}
