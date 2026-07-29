use std::io::Write;
use std::path::{Path, PathBuf};

use rstest::rstest;

pub mod testing;

#[ctor::ctor(unsafe)]
fn init() {
    env_logger::init();
}

#[rstest]
fn tags_default(
    #[base_dir = "../testdata/tags"]
    #[files("*.ledger")]
    input: PathBuf,
) {
    println!("test input file path: {}", input.display());
    let golden = golden_of(&input, "golden.tags.default.txt");

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args(["tags".as_ref(), input.as_os_str()])
        .assert()
        .success();

    assert_golden(golden, result);
}

#[rstest]
fn tags_with_values(
    #[base_dir = "../testdata/tags"]
    #[files("*.ledger")]
    input: PathBuf,
) {
    println!("test input file path: {}", input.display());
    let golden = golden_of(&input, "golden.tags.values.txt");

    let result = assert_cmd::Command::new(&*testing::BIN_PATH)
        .args(["tags".as_ref(), input.as_os_str(), "--values".as_ref()])
        .assert()
        .success();

    assert_golden(golden, result);
}

fn golden_of(input: &Path, extension: &str) -> okane_golden::Golden {
    let mut golden_path = input.to_path_buf();
    let filename = golden_path.file_name().unwrap().to_owned();
    assert!(golden_path.pop());
    golden_path.push("golden");
    golden_path.push(filename);
    assert!(
        golden_path.set_extension(extension),
        "failed to set extension .ledger to input {}",
        input.display()
    );
    log::info!("golden_path: {}", golden_path.display());
    okane_golden::Golden::new(golden_path).unwrap()
}

fn assert_golden(golden: okane_golden::Golden, result: assert_cmd::assert::Assert) {
    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    golden.assert(stdout);
}
