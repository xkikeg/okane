use okane::cmd;
use okane::import::ImportError;

use std::path::PathBuf;

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
    Import {
        #[clap(short, long, parse(from_os_str), value_name = "FILE")]
        config: PathBuf,

        source: PathBuf,
    },
}

impl Cli {
    fn run(self) -> Result<(), ImportError> {
        match self.command {
            Command::Import { config, source } => cmd::ImportCmd {
                config_path: config.as_ref(),
                target_path: source.as_ref(),
            }
            .run(&mut std::io::stdout().lock()),
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
