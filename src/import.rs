// Define converter interface here.

pub mod config;
pub mod csv;
pub mod iso_camt053;
pub mod single_entry;

mod error;

pub use error::ImportError;

use crate::data::Transaction;

/// Format of the supported importer.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Format {
    CSV,
    IsoCamt053,
}

///
pub fn import<R: std::io::Read>(
    r: &mut R,
    fmt: Format,
    config: &config::ConfigEntry,
) -> Result<Vec<Transaction>, ImportError> {
    let txns = match fmt {
        Format::CSV => csv::CSVImporter {}.import(r, config),
        // TODO: implement this.
        Format::IsoCamt053 => Err(ImportError::UnknownFormat),
    }?;
    let mut res = Vec::new();
    for txn in txns {
        let de = txn.to_double_entry(config.account.as_str())?;
        res.push(de);
    }
    return Ok(res);
}

/// Trait the each format should implement, to be used in import() internally.
pub trait Importer {
    fn import<R: std::io::Read>(
        &self,
        r: &mut R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError>;
}
