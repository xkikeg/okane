// Define converter interface here.

pub mod config;
pub mod csv;
pub mod iso_camt053;

mod error;

pub use error::ImportError;

use crate::data::Transaction;

pub trait Importer {
    fn import<R: std::io::Read>(&self, r: &mut R, config: &config::ConfigEntry) -> Result<Vec<Transaction>, ImportError>;
}
