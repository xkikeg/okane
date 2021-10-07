use crate::import::{self, Format, ImportError};

use std::ffi::OsStr;
use std::fs::File;

use encoding_rs_io::DecodeReaderBytesBuilder;

pub struct ImportCmd<'c> {
    pub config_path: &'c std::path::Path,
    pub target_path: &'c std::path::Path,
}

impl<'c> ImportCmd<'c> {
    pub fn run<W>(&self, w: &mut W) -> Result<(), ImportError>
    where
        W: std::io::Write,
    {
        let config_file = File::open(self.config_path)?;
        let config_set = import::config::load_from_yaml(config_file)?;
        let config_entry = config_set
            .select(self.target_path)
            .ok_or(ImportError::Other(format!(
                "config matching {} not found",
                self.target_path.display()
            )))?;
        let file = File::open(&self.target_path)?;
        // Use dedicated flags or config systems instead.
        let format = match self.target_path.extension().and_then(OsStr::to_str) {
            Some("csv") => Ok(Format::CSV),
            Some("xml") => Ok(Format::IsoCamt053),
            _ => Err(ImportError::UnknownFormat),
        }?;
        let mut decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(config_entry.encoding.as_encoding()))
            .build(file);
        let xacts = import::import(&mut decoded, format, config_entry)?;
        for xact in &xacts {
            writeln!(w, "{}", xact)?;
        }
        return Ok(());
    }
}
