use std::path::PathBuf;

use rstest::rstest;

pub mod testing;

#[ctor::ctor(unsafe)]
fn init() {
    env_logger::init();
}

#[rstest]
fn test_import_with_cli(
    #[base_dir = "../testdata/import/"]
    #[files("*.csv")]
    #[files("*.tsv")]
    #[files("iso_camt.xml")]
    #[files("viseca.txt")]
    input: PathBuf,
) {
    let config = testing::TESTDATA_DIR.join("import/test_config.yml");
    let mut golden_path = input.clone();
    assert!(
        golden_path.set_extension("ledger"),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    let golden = okane_golden::Golden::new(golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "import".as_ref(),
            "--rules".as_ref(),
            config.as_os_str(),
            input.as_os_str(),
        ])
        .assert()
        .success();

    let output = result.get_output();
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}

#[rstest]
#[case::long("--config")]
#[case::short("-c")]
fn test_import_deprecated_config_flag_still_works(#[case] flag: &str) {
    let config = testing::TESTDATA_DIR.join("import/test_config.yml");
    let input = testing::TESTDATA_DIR.join("import/csv_delimiter.csv");
    let golden = okane_golden::Golden::new(input.with_extension("ledger")).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "import".as_ref(),
            flag.as_ref(),
            config.as_os_str(),
            input.as_os_str(),
        ])
        .assert()
        .success();

    let output = result.get_output();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        stderr.contains("`--config` / `-c` is deprecated, use `--rules` / `-r` instead"),
        "missing deprecation warning in stderr: {stderr}"
    );
    golden.assert(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_import_rejects_both_rules_and_config() {
    let config = testing::TESTDATA_DIR.join("import/test_config.yml");
    let input = testing::TESTDATA_DIR.join("import/csv_delimiter.csv");

    assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "import".as_ref(),
            "--rules".as_ref(),
            config.as_os_str(),
            "--config".as_ref(),
            config.as_os_str(),
            input.as_os_str(),
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_import_requires_rules() {
    let input = testing::TESTDATA_DIR.join("import/csv_delimiter.csv");

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args(["import".as_ref(), input.as_os_str()])
        .assert()
        .failure()
        .code(2);

    let stderr = std::str::from_utf8(&result.get_output().stderr).unwrap();
    assert!(
        stderr.contains("--rules"),
        "error should name --rules: {stderr}"
    );
    assert!(
        !stderr.contains("--config"),
        "error should not advertise the deprecated flag: {stderr}"
    );
}
