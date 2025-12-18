//! Simple golden testing framework.
//!
//! You can simply create a golden file by calling [`Golden::new`].
//!
//! ```
//! let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/mukai.txt");
//! let golden = okane_golden::Golden::new(target)?;
//! golden.assert("zazen boys\n");
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! If the test fails, simply pass `UPDATE_GOLDEN=1` env var to rerun the test.

use std::path::{Path, PathBuf};

use pretty_assertions::assert_str_eq;

/// Golden object to maintain the expected content.
#[derive(Debug)]
pub struct Golden {
    path: PathBuf,
    content: String,
}

fn is_update_golden() -> bool {
    !std::env::var("UPDATE_GOLDEN")
        .unwrap_or_default()
        .is_empty()
}

impl Golden {
    /// Returns a new instance.
    /// Note returned instance ignores CR/CRLF difference.
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let content = read_as_utf8(&path).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                if is_update_golden() {
                    Ok(String::new())
                } else {
                    Err(std::io::Error::new(e.kind(), format!("Golden file {} not found: pass the environment variable UPDATE_GOLDEN=1 to write golden file", path.display())))
                }
            } else {
                Err(e)
            }
        })?;
        Ok(Self { path, content })
    }

    /// Assert the given `got` str against the golden file.
    /// Pass `UPDATE_GOLDEN` env to update the golden itself.
    pub fn assert(&self, got: &str) {
        let want;
        if is_update_golden() {
            want = got;
            // update the original_file.
            std::fs::write(&self.path, got).expect("Update golden failed");
        } else {
            want = &self.content;
        }
        assert_str_eq!(
            want,
            got,
            "comparison against golden failed. Pass UPDATE_GOLDEN=1 to update the golden."
        );
    }
}

/// Returns content of testdata directory, as UTF-8.
/// This fucntion replaces CRLF to LF.
fn read_as_utf8(filename: &Path) -> std::io::Result<String> {
    // Needs to replace CRLF to LF for Windows, as all files may have CRLF
    // depending on the git config core.autocrlf.
    std::fs::read_to_string(filename).map(|s| s.replace("\r\n", "\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use regex::Regex;

    fn testdata_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/")
    }

    #[test]
    fn new_fails_on_non_existing_file() {
        temp_env::with_var_unset("UPDATE_GOLDEN", || {
            let path = testdata_dir().join("non_existing_golden.txt");
            let got_err = Golden::new(path).expect_err("this must fail");

            assert!(Regex::new("Golden file .* not found")
                .unwrap()
                .is_match(&got_err.to_string()));
        });
    }

    #[test]
    fn assert_succeeds_on_correct_golden() {
        temp_env::with_var_unset("UPDATE_GOLDEN", || {
            let path = testdata_dir().join("mukai.txt");

            let golden = Golden::new(path).unwrap();
            golden.assert("zazen boys\n");
        });
    }

    #[test]
    fn assert_fails_on_different_content() {
        temp_env::with_var_unset("UPDATE_GOLDEN", || {
            let path = testdata_dir().join("mukai.txt");

            let golden = Golden::new(path).unwrap();

            let got_err = std::panic::catch_unwind(|| golden.assert("number girl"))
                .expect_err("this assertion must fail");

            let payload = &*got_err;
            if payload.is::<String>() {
                assert!(payload
                    .downcast_ref::<String>()
                    .unwrap()
                    .contains("assertion failed"));
            } else {
                panic!("unexpected type of assertion failure");
            }
        });
    }
}
