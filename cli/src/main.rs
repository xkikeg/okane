//! okane is a tool to have a plain text accounting, inspired by [Ledger CLI](https://ledger-cli.org/).
//!
//! For the detailed usage, see [README](https://github.com/xkikeg/okane/blob/main/README.md) or
//! [Japanese README](https://github.com/xkikeg/okane/blob/main/README.md).
//!
//! As a library, [`okane-core`](https://crates.io/crates/okane-core) provides reusable functionalities.
//! As oppose to that, this library mainly provides binary specific functionalities, mainly for integration tests.

mod cmd;
mod config;
mod format;
mod import;
#[cfg(test)]
mod one_based_macro;
mod ui;

use shadow_rs::shadow;

shadow!(build);

use clap::Parser as _;

fn main() {
    env_logger::init();
    let cli = cmd::Cli::parse();
    let config = if cli.no_config() {
        None
    } else {
        match load_config() {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("{}", err);
                for cause in err.chain().skip(1) {
                    eprintln!("Caused by {}", cause);
                }
                std::process::exit(2);
            }
        }
    };
    let command = match cli.build(config.as_ref()) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(2);
        }
    };
    if let Err(err) = command.run(&mut std::io::stdout().lock()) {
        eprintln!("{}", err);
        for cause in err.chain().skip(1) {
            eprintln!("Caused by {}", cause);
        }
        std::process::exit(1);
    }
}

fn load_config() -> anyhow::Result<config::Config> {
    use anyhow::Context as _;
    let env = config::Env::from_process().context("failed to read the current directory")?;
    Ok(config::load(&env, &config::RealFs)?)
}

#[cfg(test)]
#[ctor::ctor(unsafe)]
fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::max())
        .try_init();
}
