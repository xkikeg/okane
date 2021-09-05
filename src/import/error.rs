use std::error;
use std::fmt;

use ::csv::Error as CsvError;

#[derive(Debug)]
pub enum ImportError {
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

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImportError::IO(_) => write!(f, "failed to perform IO"),
            ImportError::CSV(_) => write!(f, "failed to parse CSV"),
            ImportError::XML(_) => write!(f, "failed to parse XML"),
            ImportError::InvalidFlag(name) => write!(f, "invalid flag {}", name),
            // ImportError::Unknown(x) =>
            //   write!(f, "unknown error"),
            ImportError::InvalidDatetime(_) => write!(f, "invalid datetime"),
            ImportError::InvalidDecimal(_) => write!(f, "invalid decimal"),
            ImportError::Other(ref x) => write!(f, "other: {}", x),
            ImportError::UnknownFormat => write!(f, "unknown format"),
        }
    }
}

impl error::Error for ImportError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ImportError::IO(ref e) => Some(e),
            ImportError::CSV(ref e) => Some(e),
            ImportError::XML(ref e) => Some(e),
            ImportError::InvalidFlag(_) => None,
            // ImportError::Unknown(e) => Some(&e),
            ImportError::InvalidDatetime(ref e) => Some(e),
            ImportError::InvalidDecimal(ref e) => Some(e),
            ImportError::Other(_) => None,
            ImportError::UnknownFormat => None,
        }
    }
}

impl From<std::io::Error> for ImportError {
    fn from(err: std::io::Error) -> ImportError {
        ImportError::IO(err)
    }
}

impl From<chrono::ParseError> for ImportError {
    fn from(err: chrono::ParseError) -> ImportError {
        ImportError::InvalidDatetime(err)
    }
}

impl From<CsvError> for ImportError {
    fn from(err: CsvError) -> ImportError {
        ImportError::CSV(err)
    }
}

impl From<quick_xml::DeError> for ImportError {
    fn from(err: quick_xml::DeError) -> ImportError {
        ImportError::XML(err)
    }
}

impl From<rust_decimal::Error> for ImportError {
    fn from(err: rust_decimal::Error) -> ImportError {
        ImportError::InvalidDecimal(err)
    }
}
