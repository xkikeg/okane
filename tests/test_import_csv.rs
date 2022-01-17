mod testing;

use pretty_assertions::assert_eq;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[test]
fn test_import_csv_index() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("index_amount.csv");
    let want = testing::read_as_utf8("index_amount.ledger").expect("cannot read want");
    let mut result: Vec<u8> = Vec::new();

    okane::cmd::ImportCmd {
        config_path: &config,
        target_path: &input,
    }
    .run(&mut result)
    .expect("execution failed");

    let got = String::from_utf8(result).expect("invalid UTF-8");
    assert_eq!(want, got);
}

#[test]
fn test_import_csv_label() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("label_credit_debit.csv");
    let want = testing::read_as_utf8("label_credit_debit.ledger").expect("cannot read want");

    let mut result: Vec<u8> = Vec::new();
    okane::cmd::ImportCmd {
        config_path: &config,
        target_path: &input,
    }
    .run(&mut result)
    .expect("execution failed");
    let got = String::from_utf8(result).expect("invalid UTF-8");
    assert_eq!(want, got);
}

#[test]
fn test_import_csv_multi_currency() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("csv_multi_currency.csv");
    let want = testing::read_as_utf8("csv_multi_currency.ledger").expect("cannot read want");

    let mut result: Vec<u8> = Vec::new();
    okane::cmd::ImportCmd {
        config_path: &config,
        target_path: &input,
    }
    .run(&mut result)
    .expect("execution failed");
    let got = String::from_utf8(result).expect("invalid UTF-8");
    assert_eq!(want, got);
}
