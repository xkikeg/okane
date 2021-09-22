use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;

use okane::import::ImportError;
use okane::import::Format;

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
    let config_file = File::open(Path::new(&args[1]))?;
    let config_set = okane::import::config::load_from_yaml(config_file)?;
    let config_entry = config_set.entries.get(0).unwrap();
    let path = Path::new(&args[2]);
    let file = File::open(&path)?;
    // Use dedicated flags or config systems instead.
    let format = match path.extension().and_then(OsStr::to_str) {
        Some("csv") => Ok(Format::CSV),
        Some("xml") => Ok(Format::IsoCamt053),
        _ => Err(ImportError::UnknownFormat),
    }?;
    let encoding = match format {
        Format::CSV => "Shift_JIS",
        Format::IsoCamt053 => "UTF-8",
    };
    let character_encoder = encoding_rs::Encoding::for_label(encoding.as_bytes())
        .ok_or(ImportError::InvalidFlag("--encoding"))?;
    let mut decoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(character_encoder))
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
