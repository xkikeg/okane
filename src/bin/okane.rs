use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use okane::converter::ConvertError;
use encoding_rs_io::DecodeReaderBytesBuilder;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        std::process::exit(1);
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
            let character_encoder = encoding_rs::Encoding::for_label(b"UTF-8")
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
