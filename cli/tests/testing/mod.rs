use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TESTDATA_DIR: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("testdata");
    pub static ref BIN_PATH: PathBuf = assert_cmd::cargo::cargo_bin!().to_path_buf();
}

/// Command running okane without reading any config file, so that the
/// config of whoever runs the tests can't change the results.
pub fn okane() -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::new(&*BIN_PATH);
    cmd.arg("--no-config");
    cmd
}
