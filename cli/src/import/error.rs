use csv::Error as CsvError;
use regex::Error as RegexError;

use super::template;

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse CSV")]
    Csv(#[from] CsvError),
    #[error("failed to parse XML")]
    Xml(#[from] quick_xml::DeError),
    #[error("failed to parse YAML")]
    Yaml(#[from] serde_yaml::Error),
    #[error("failed to parse VISECA file: {0}")]
    Viseca(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("invalid datetime")]
    InvalidDatetime(#[from] chrono::ParseError),
    #[error("invalid decimal")]
    InvalidDecimal(#[from] rust_decimal::Error),
    #[error("invalid regex")]
    InvalidRegex(#[from] RegexError),
    #[error("failed to parse template")]
    TemplateParseFailed(#[from] template::ParseError),
    #[error("failed to render template")]
    TemplateRenderFailed(#[from] template::RenderError),
    #[error("other error: {0}")]
    Other(String),
    #[error("unimplemented: {0}")]
    Unimplemented(&'static str),
    #[error("unknown format")]
    UnknownFormat,
}
