use clap::Parser as _;

use okane::cmd;

fn main() {
    env_logger::init();
    let cli = cmd::Cli::parse();
    if let Err(err) = cli.run(&mut std::io::stdout().lock()) {
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
