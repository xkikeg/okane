use core::str;
use std::{fmt::Display, path::PathBuf};

use clap::Parser as _;
use okane::cmd;
use rstest::rstest;

pub mod testing;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

fn print_err<E: Display>(x: E) -> E {
    eprintln!("{}", x);
    x
}

#[rstest]
fn balance_default(
    #[base_dir = "../testdata/report"]
    #[files("*.ledger")]
    input: PathBuf,
) {
    let mut golden_path = input.clone();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension("golden.balance.default.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = testing::Golden::new(golden_path).unwrap();

    let args = shlex::split(&format!("binary balance {}", input.display())).unwrap();

    let cli = cmd::Cli::try_parse_from(&args).map_err(print_err).unwrap();
    let mut got: Vec<u8> = Vec::new();

    cli.run(&mut got).map_err(print_err).unwrap();

    golden.assert(str::from_utf8(&got).unwrap());
}
