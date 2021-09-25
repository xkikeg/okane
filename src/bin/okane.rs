use okane::cmd;
use okane::import::Format;
use okane::import::ImportError;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;

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
    let config_entry = config_set.select(path).ok_or(ImportError::Other(format!(
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
    match format {
        Format::CSV => {
            return cmd::ImportCmd {
                config_path: &config_path,
                target_path: &path,
            }
            .run(&mut std::io::stdout().lock());
        }
        Format::IsoCamt053 => {
            let decoded = DecodeReaderBytesBuilder::new()
                .encoding(Some(config_entry.encoding.as_encoding()))
                .build(file);
            let res = okane::import::iso_camt053::print_camt(std::io::BufReader::new(decoded))?;
            println!("{}", res);
            return Ok(());
        }
    }
}
