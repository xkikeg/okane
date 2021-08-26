extern crate csv;
extern crate encoding_rs_io;
extern crate quick_xml;

use std::env;
use std::error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

#[derive(Debug)]
enum ConvertError {
    IO(std::io::Error),
    CSV(csv::Error),
    XML(quick_xml::DeError),
    InvalidFlag(&'static str),
    // Unknown(Box<dyn std::error::Error>),
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

impl From<csv::Error> for ConvertError {
    fn from(err: csv::Error) -> ConvertError {
        ConvertError::CSV(err)
    }
}

impl From<quick_xml::DeError> for ConvertError {
    fn from(err: quick_xml::DeError) -> ConvertError {
        ConvertError::XML(err)
    }
}

fn try_main() -> Result<(), ConvertError> {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let file = File::open(&path)?;
    // Use dedicated flags or config systems instead.
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => {
            let character_encoder = encoding_rs::Encoding::for_label(b"Shift_JIS")
                .ok_or(ConvertError::InvalidFlag("--encoding"))?;
            let decoded = DecodeReaderBytesBuilder::new()
                .encoding(Some(character_encoder))
                .build(file);
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(b',')
                .from_reader(decoded);
            for result in rdr.records() {
                let r = result?;
                println!("{:?}", r);
            }
            return Ok(());
        }
        Some("xml") => {
            let character_encoder = encoding_rs::Encoding::for_label(b"UTF_8")
                .ok_or(ConvertError::InvalidFlag("--encoding"))?;
            let decoded = DecodeReaderBytesBuilder::new()
                .encoding(Some(character_encoder))
                .build(file);
            let res = okane::converter::iso_camt053::print_camt(std::io::BufReader::new(decoded))?;
            println!("{}", res);
            return Ok(());
        }
        _ => Err(ConvertError::UnknownFormat),
    }
}
