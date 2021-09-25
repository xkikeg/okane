use std::io::{self, Write};
use std::path::{Path, PathBuf};

use indoc::indoc;
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;

lazy_static! {
    static ref TESTDATA_DIR: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/testdata");
    static ref BIN_PATH: PathBuf = assert_cmd::cargo::cargo_bin(assert_cmd::crate_name!());
}

static IMPORT_CSV_INDEX_WANT: &str = indoc! {r#"
2021/09/01 * cashback
    Incomes:Misc                              -50.00 USD
    Liabilities:Okane Card                     50.00 USD

2021/09/02 * (31415) Migros
    Liabilities:Okane Card                    -28.00 USD
    Expenses:Grocery                           28.00 USD

2021/09/03 * (14142) FooBar
    Liabilities:Okane Card                     -1.45 USD
    ! Expenses:Unknown                          1.45 USD


"#};

#[test]
fn test_import_csv_index() {
    let config = TESTDATA_DIR.join("test_config.yml");
    let input = TESTDATA_DIR.join("index_amount.csv");

    // Test with code invocation
    {
        let mut result: Vec<u8> = Vec::new();
        okane::cmd::ImportCmd {
            config_path: &config,
            target_path: &input,
        }
        .run(&mut result)
        .expect("execution failed");
        let got = String::from_utf8(result).expect("invalid UTF-8");
        assert_eq!(IMPORT_CSV_INDEX_WANT, got);
    }
    // Test with cmd execution
    {
        let result = assert_cmd::Command::new(&*BIN_PATH)
            .args(&[config, input])
            .assert()
            .success();
        let output = result.get_output();
        io::stderr().write_all(&output.stderr).unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert_eq!(IMPORT_CSV_INDEX_WANT, stdout);
    }
}

static IMPORT_CSV_LABEL_WANT: &str = indoc! {r#"
    2021/09/01 * 五反田ATM
        Assets:Cash                                -5000 JPY
        Assets:Okane Bank                           5000 JPY = 12345 JPY

    2021/09/02 * (31415) Migros
        Assets:Okane Bank                          -2800 JPY = 9545 JPY
        Expenses:Grocery                            2800 JPY

    2021/09/03 * (14142) FooBar
        Assets:Okane Bank                           -145 JPY = 9400 JPY
        ! Expenses:Unknown                           145 JPY


"#};

#[test]
fn test_import_csv_label() {
    let config = TESTDATA_DIR.join("test_config.yml");
    let input = TESTDATA_DIR.join("label_credit_debit.csv");

    // Test with code invocation
    {
        let mut result: Vec<u8> = Vec::new();
        okane::cmd::ImportCmd {
            config_path: &config,
            target_path: &input,
        }
        .run(&mut result)
        .expect("execution failed");
        let got = String::from_utf8(result).expect("invalid UTF-8");
        assert_eq!(IMPORT_CSV_LABEL_WANT, got);
    }
    // Test with command invocation
    {
        let result = assert_cmd::Command::new(&*BIN_PATH)
            .args(&[config, input])
            .assert()
            .success();
        let output = result.get_output();
        io::stderr().write_all(&output.stderr).unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert_eq!(IMPORT_CSV_LABEL_WANT, stdout);
    }
}
