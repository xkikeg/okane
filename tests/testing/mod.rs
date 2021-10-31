use std::path::{Path, PathBuf};

use std::fs;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TESTDATA_DIR: PathBuf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/testdata");
    pub static ref BIN_PATH: PathBuf = assert_cmd::cargo::cargo_bin(assert_cmd::crate_name!());
}

/// Returns content of testdata directory, as UTF-8.
pub fn read_as_utf8(filename: &str) -> std::io::Result<String> {
    fs::read_to_string(TESTDATA_DIR.join(filename))
}
