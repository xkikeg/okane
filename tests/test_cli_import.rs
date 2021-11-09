mod testing;

use std::io::{self, Write};

use pretty_assertions::assert_eq;

#[test]
fn test_import_success() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("index_amount.csv");
    let want = testing::read_as_utf8("index_amount.ledger").unwrap();

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args(&[config, input])
        .assert()
        .success();

    let output = result.get_output();
    io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(want, stdout);
}
