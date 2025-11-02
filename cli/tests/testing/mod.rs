use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TESTDATA_DIR: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("testdata");
    pub static ref BIN_PATH: PathBuf = assert_cmd::cargo::cargo_bin!().to_path_buf();
}
