pub mod testing;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[test]
fn test_import_csv_index() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("index_amount.csv");
    let golden = testing::Golden::new("index_amount.ledger").expect("cannot create golden");
    let mut result: Vec<u8> = Vec::new();

    okane::cmd::ImportCmd {
        config,
        source: input,
    }
    .run(&mut result)
    .expect("execution failed");
    let got = String::from_utf8(result).expect("invalid UTF-8");

    golden.assert(&got);
}

#[test]
fn test_import_csv_label() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("label_credit_debit.csv");
    let golden = testing::Golden::new("label_credit_debit.ledger").expect("cannot create golden");

    let mut result: Vec<u8> = Vec::new();
    okane::cmd::ImportCmd {
        config,
        source: input,
    }
    .run(&mut result)
    .expect("execution failed");
    let got = String::from_utf8(result).expect("invalid UTF-8");

    golden.assert(&got);
}

#[test]
fn test_import_csv_multi_currency() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("csv_multi_currency.csv");
    let golden = testing::Golden::new("csv_multi_currency.ledger").expect("cannot create golden");

    let mut result: Vec<u8> = Vec::new();
    okane::cmd::ImportCmd {
        config,
        source: input,
    }
    .run(&mut result)
    .expect("execution failed");
    let got = String::from_utf8(result).expect("invalid UTF-8");

    golden.assert(&got);
}

#[test]
fn test_import_csv_template() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("csv_template.csv");
    let golden = testing::Golden::new("csv_template.ledger").expect("cannot create golden");

    let mut result: Vec<u8> = Vec::new();
    okane::cmd::ImportCmd {
        config,
        source: input,
    }
    .run(&mut result)
    .expect("execution failed");
    let got = String::from_utf8(result).expect("invalid UTF-8");

    golden.assert(&got);
}
