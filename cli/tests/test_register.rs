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

#[rstest]
fn register_cel_filter(
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
        golden_path.set_extension("golden.register.cel_filter.txt"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    // Bank postings of the cleared transactions that also touch an expense.
    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
            input.as_os_str(),
            OsStr::new("--filter"),
            OsStr::new(
                r#"account.startsWith("Assets:Banks:") && t.state == "cleared" && t.postings.exists(p, p.account.startsWith("Expenses:"))"#,
            ),
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
fn register_cel_filter_invalid(
    #[base_dir = "../testdata/report"]
    #[files("single_commodity.ledger")]
    input: PathBuf,
) {
    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
            input.as_os_str(),
            OsStr::new("--filter"),
            OsStr::new("payee =="),
        ])
        .assert()
        .failure();

    let stderr = std::str::from_utf8(&result.get_output().stderr).unwrap();
    assert!(stderr.contains("--filter"), "stderr: {stderr}");
}

#[rstest]
#[case::tag_value("trip_kyoto", r#"tags[?"trip"].orValue("") == "kyoto""#)]
#[case::tag_presence(
    "business",
    r#""business" in t.tags && account.startsWith("Expenses:")"#
)]
#[case::other_posting_tag(
    "bank_of_osaka_txn",
    r#"account == "Assets:Banks:Main" && t.postings.exists(p, p.tags[?"trip"] == optional.of("osaka"))"#
)]
fn register_cel_filter_tags(#[case] name: &str, #[case] filter: &str) {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../testdata/filter/tagged.ledger");
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../testdata/filter/golden/tagged.register.{name}.txt"
    ));
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "register".as_ref(),
            input.as_os_str(),
            OsStr::new("--filter"),
            OsStr::new(filter),
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}
