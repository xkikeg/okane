mod testing;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[test]
fn test_import() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("iso_camt.xml");
    let want = testing::read_as_utf8("iso_camt.ledger").expect("cannot read want");
    let mut result: Vec<u8> = Vec::new();

    okane::cmd::ImportCmd {
        config,
        source: input,
    }
    .run(&mut result)
    .expect("execution failed");

    let got = String::from_utf8(result).expect("invalid UTF-8");
    if want != got {
        panic!(
            "unexpected output: diff (-want +got)\n{}",
            colored_diff::PrettyDifference {
                expected: want.as_str(),
                actual: got.as_str()
            }
        )
    }
}
