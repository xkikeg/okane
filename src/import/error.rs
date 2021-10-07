use ::csv::Error as CsvError;

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse CSV")]
    CSV(#[from] CsvError),
    #[error("failed to parse XML")]
    XML(#[from] quick_xml::DeError),
    #[error("failed to parse YAML")]
    YAML(#[from] serde_yaml::Error),
    #[error("invalid flag {0}")]
    InvalidFlag(&'static str),
    #[error("invalid config {0}")]
    InvalidConfig(&'static str),
    #[error("invalid datetime")]
    InvalidDatetime(#[from] chrono::ParseError),
    #[error("invalid decimal")]
    InvalidDecimal(#[from] rust_decimal::Error),
    #[error("other error: {0}")]
    Other(String),
    #[error("unknown format")]
    UnknownFormat,
}
