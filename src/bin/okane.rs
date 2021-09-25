use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;

use okane::import::Format;
use okane::import::ImportError;

fn main() {
    env_logger::init();
    if let Err(err) = try_main() {
        use std::error::Error;
        eprintln!("{}", err);
        if let Some(src) = err.source() {
            eprintln!("Caused by {}", src);
        }
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), ImportError> {
    let args: Vec<String> = env::args().collect();
    let config_path = Path::new(&args[1]);
    let path = Path::new(&args[2]);
    let config_file = File::open(config_path)?;
    let config_set = okane::import::config::load_from_yaml(config_file)?;
    let config_entry = config_set
        .select(path)
        .ok_or(ImportError::Other(format!(
            "config matching {} not found",
            path.display()
        )))?;
    let file = File::open(&path)?;
    // Use dedicated flags or config systems instead.
    let format = match path.extension().and_then(OsStr::to_str) {
        Some("csv") => Ok(Format::CSV),
        Some("xml") => Ok(Format::IsoCamt053),
        _ => Err(ImportError::UnknownFormat),
    }?;
    let mut decoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(config_entry.encoding.as_encoding()))
        .build(file);
    match format {
        Format::CSV => {
            let xacts = okane::import::import(&mut decoded, format, config_entry)?;
            for xact in &xacts {
                println!("{}", xact);
            }
            return Ok(());
        }
        Format::IsoCamt053 => {
            let res = okane::import::iso_camt053::print_camt(std::io::BufReader::new(decoded))?;
            println!("{}", res);
            return Ok(());
        }
    }
}
