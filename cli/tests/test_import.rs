use std::path::PathBuf;

use pretty_assertions::assert_eq;
use rstest::rstest;

pub mod testing;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[rstest]
fn test_import_with_cli(
    #[base_dir = "../testdata/import/"]
    #[files("*.csv")]
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
    let want = okane_golden::read_as_utf8(&golden_path).unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args([
            "import".as_ref(),
            "--config".as_ref(),
            config.as_os_str(),
            input.as_os_str(),
        ])
        .assert()
        .success();

    let output = result.get_output();
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(want, stdout);
}
