use std::error::Error;

use clap::Parser as _;

use okane::cmd;

fn main() {
    env_logger::init();
    let cli = cmd::Cli::parse();
    if let Err(err) = cli.validate() {
        eprintln!("{}", err);
        std::process::exit(2);
    }
    if let Err(err) = cli.run(&mut std::io::stdout().lock()) {
        eprintln!("{}", err);
        let mut cur: &dyn Error = &err;
        while let Some(src) = cur.source() {
            eprintln!("Caused by {}", src);
            cur = src;
        }
        std::process::exit(1);
    }
}
