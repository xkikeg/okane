use okane::cmd;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Cli {
    #[clap(subcommand)]
    command: cmd::Command,
}
impl Cli {
    fn run(self) -> Result<(), cmd::Error> {
        self.command.run()
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
