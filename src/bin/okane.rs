use okane::cmd;
use okane::import::ImportError;

use std::env;
use std::path::Path;

fn main() {
    env_logger::init();
    if let Err(err) = try_main() {
        use std::error::Error;
        eprintln!("{}", err);
        let mut cur: &dyn Error = &err;
        while let Some(src) = cur.source() {
            eprintln!("Caused by {}", src);
            cur = src;
        }
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), ImportError> {
    let args: Vec<String> = env::args().collect();
    let config_path = Path::new(&args[1]);
    let target_path = Path::new(&args[2]);
    return cmd::ImportCmd {
        config_path,
        target_path,
    }
    .run(&mut std::io::stdout().lock());
}
