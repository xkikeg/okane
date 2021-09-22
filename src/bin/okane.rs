use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;
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

enum Format {
    CSV,
    IsoCamt,
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
        Some("xml") => Ok(Format::IsoCamt),
        _ => Err(ImportError::UnknownFormat),
    }?;
    let encoding = match format {
        Format::CSV => "Shift_JIS",
        Format::IsoCamt => "UTF-8",
    };
    let character_encoder = encoding_rs::Encoding::for_label(encoding.as_bytes())
        .ok_or(ImportError::InvalidFlag("--encoding"))?;
    let mut decoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(character_encoder))
        .build(file);
    match format {
        Format::CSV => {
            use okane::import::Importer;
            let c = okane::import::csv::CSVImporter{};
            let xacts = c.import(&mut decoded, config_entry)?;
            for xact in &xacts {
                println!("{}", xact);
            }
            return Ok(());
        }
        Format::IsoCamt => {
            let res = okane::import::iso_camt053::print_camt(std::io::BufReader::new(decoded))?;
            println!("{}", res);
            return Ok(());
        }
    }
}
