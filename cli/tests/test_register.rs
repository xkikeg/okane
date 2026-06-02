use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;

use rstest::rstest;

pub mod testing;

#[ctor::ctor(unsafe)]
fn init() {
    env_logger::init();
}

#[rstest]
fn register_default(
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
        golden_path.set_extension("golden.register.default.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args(["register".as_ref(), input.as_os_str()])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn register_exact_account_filter(
    #[base_dir = "../testdata/report"]
    #[files("single_commodity.ledger")]
    input: PathBuf,
) {
    let mut golden_path = input.clone();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension("golden.register.account_filter_exact.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
            input.as_os_str(),
            OsStr::new("--account-filter=exact"),
            OsStr::new("Assets:Banks:あおによし"),
            OsStr::new("Expenses:Cash"),
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn register_regex_account_filter(
    #[base_dir = "../testdata/report"]
    #[files("single_commodity.ledger")]
    input: PathBuf,
) {
    let mut golden_path = input.clone();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension("golden.register.account_filter_regex.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
            input.as_os_str(),
            OsStr::new("Assets:Banks"),
            OsStr::new("^Card"), // ignored
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn register_date_range(
    #[base_dir = "../testdata/report"]
    #[files("single_commodity.ledger")]
    input: PathBuf,
) {
    let mut golden_path = input.clone();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension("golden.register.date_range.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
            input.as_os_str(),
            OsStr::new("--start"),
            OsStr::new("2024-02-01"),
            OsStr::new("--end"),
            OsStr::new("2024-02-20"),
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn register_in_usd_historical(
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
        golden_path.set_extension("golden.register.in_usd_historical.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
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
