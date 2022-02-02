use okane::cmd;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Import other format into ledger.
    Import(cmd::ImportCmd),
    /// Format the given file (in future it'll work without file arg)
    Format(cmd::FormatCmd),
}

impl Cli {
    fn run(self) -> Result<(), cmd::Error> {
        match self.command {
            Command::Import(cmd) => cmd.run(&mut std::io::stdout().lock()),
            Command::Format(cmd) => cmd.run(&mut std::io::stdout().lock()),
        }
    }
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    if let Err(err) = cli.run() {
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
