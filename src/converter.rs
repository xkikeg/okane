// Define converter interface here.

pub mod csv;
pub mod iso_camt053;

mod error;

pub use error::ConvertError;

use crate::data::Transaction;

pub trait Converter {
    fn convert<R: std::io::Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ConvertError>;
}
