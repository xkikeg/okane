use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;

use rstest::rstest;

pub mod testing;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[rstest]
fn balance_default(
    #[base_dir = "../testdata/report"]
    #[files("*.ledger")]
    input: PathBuf,
) {
    println!("test input file path: {}", input.display());
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
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args(["balance".as_ref(), input.as_os_str()])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn balance_in_jpy_up_to_date(
    #[base_dir = "../testdata/report"]
    #[files("multi_commodity.ledger")]
    input: PathBuf,
) {
    let mut golden_path = input.clone();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension("golden.balance.in_jpy_up_to_date.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "balance".as_ref(),
            input.as_os_str(),
            OsStr::new("-X"),
            OsStr::new("JPY"),
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn balance_in_usd_historical(
    #[base_dir = "../testdata/report"]
    #[files("multi_commodity.ledger")]
    input: PathBuf,
) {
    let mut golden_path = input.clone();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension("golden.balance.in_usd_historical.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "balance".as_ref(),
            input.as_os_str(),
            OsStr::new("-X"),
            OsStr::new("USD"),
            OsStr::new("--historical"),
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}
