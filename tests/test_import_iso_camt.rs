mod testing;

// 振り込み in、給与、振り込み out、Maestro、Twint

// 10/1 +1000
// 10/2 +6000
// 10/3 -1880
// 10/3 -120
// 10/4 -10.1
// 10/5 -14.15 (with charge)
// 10/6 -24.75
// 10/7 -52 (withdrawal, charge 2)
// 10/8 -437.6 (EUR witdrawal)
// ----
// 4899

#[test]
fn test_import() {
    let config = testing::TESTDATA_DIR.join("test_config.yml");
    let input = testing::TESTDATA_DIR.join("iso_camt.xml");
    let want = testing::read_as_utf8("iso_camt.ledger").expect("cannot read want");
    let mut result: Vec<u8> = Vec::new();

    okane::cmd::ImportCmd {
        config_path: &config,
        target_path: &input,
    }
    .run(&mut result)
    .expect("execution failed");

    let got = String::from_utf8(result).expect("invalid UTF-8");
    if want != got {
        panic!(
            "unexpected output: diff\n{}",
            colored_diff::PrettyDifference {
                expected: want.as_str(),
                actual: got.as_str()
            }
        )
    }
}
