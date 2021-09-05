use std::error;
use std::fmt;

use ::csv::Error as CsvError;

#[derive(Debug)]
pub enum ConvertError {
    IO(std::io::Error),
    CSV(CsvError),
    XML(quick_xml::DeError),
    InvalidFlag(&'static str),
    // Unknown(Box<dyn std::error::Error>),
    InvalidDatetime(chrono::ParseError),
    InvalidDecimal(rust_decimal::Error),
    Other(String),
    UnknownFormat,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConvertError::IO(_) => write!(f, "failed to perform IO"),
            ConvertError::CSV(_) => write!(f, "failed to parse CSV"),
            ConvertError::XML(_) => write!(f, "failed to parse XML"),
            ConvertError::InvalidFlag(name) => write!(f, "invalid flag {}", name),
            // ConvertError::Unknown(x) =>
            //   write!(f, "unknown error"),
            ConvertError::InvalidDatetime(_) => write!(f, "invalid datetime"),
            ConvertError::InvalidDecimal(_) => write!(f, "invalid decimal"),
            ConvertError::Other(ref x) => write!(f, "other: {}", x),
            ConvertError::UnknownFormat => write!(f, "unknown format"),
        }
    }
}

impl error::Error for ConvertError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ConvertError::IO(ref e) => Some(e),
            ConvertError::CSV(ref e) => Some(e),
            ConvertError::XML(ref e) => Some(e),
            ConvertError::InvalidFlag(_) => None,
            // ConvertError::Unknown(e) => Some(&e),
            ConvertError::InvalidDatetime(ref e) => Some(e),
            ConvertError::InvalidDecimal(ref e) => Some(e),
            ConvertError::Other(_) => None,
            ConvertError::UnknownFormat => None,
        }
    }
}

impl From<std::io::Error> for ConvertError {
    fn from(err: std::io::Error) -> ConvertError {
        ConvertError::IO(err)
    }
}

impl From<chrono::ParseError> for ConvertError {
    fn from(err: chrono::ParseError) -> ConvertError {
        ConvertError::InvalidDatetime(err)
    }
}

impl From<CsvError> for ConvertError {
    fn from(err: CsvError) -> ConvertError {
        ConvertError::CSV(err)
    }
}

impl From<quick_xml::DeError> for ConvertError {
    fn from(err: quick_xml::DeError) -> ConvertError {
        ConvertError::XML(err)
    }
}

impl From<rust_decimal::Error> for ConvertError {
    fn from(err: rust_decimal::Error) -> ConvertError {
        ConvertError::InvalidDecimal(err)
    }
}
