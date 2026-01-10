//! okane is a tool to have a plain text accounting, inspired by [Ledger CLI](https://ledger-cli.org/).
//!
//! For the detailed usage, see [README](https://github.com/xkikeg/okane/blob/main/README.md) or
//! [Japanese README](https://github.com/xkikeg/okane/blob/main/README.md).
//!
//! As a library, [`okane-core`](https://crates.io/crates/okane-core) provides reusable functionalities.
//! As oppose to that, this library mainly provides binary specific functionalities, mainly for integration tests.

mod cmd;
mod format;
mod import;
#[cfg(test)]
mod one_based_macro;

use shadow_rs::shadow;

shadow!(build);

use std::error::Error;

use clap::Parser as _;

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
