use std::path::{Path, PathBuf};

use pretty_assertions::assert_str_eq;

/// Returns content of testdata directory, as UTF-8.
/// This fucntion replaces CRLF to LF.
pub fn read_as_utf8(filename: &Path) -> std::io::Result<String> {
    // Needs to replace CRLF to LF for Windows, as all files may have CRLF
    // depending on the git config core.autocrlf.
    std::fs::read_to_string(filename).map(|s| s.replace("\r\n", "\n"))
}

/// Golden file, which maintains the expected content and assert over that.
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
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let content = read_as_utf8(&path).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                if is_update_golden() {
                    Ok(String::new())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Golden file {} not found: pass the environment variable UPDATE_GOLDEN=1", path.display())))
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

#[cfg(test)]
mod tests {
    use super::*;

    use test_temp_dir::test_temp_dir;

    mod update_golden_unset {
        use regex::Regex;

        use super::*;

        #[test]
        fn new_fails_on_non_existing_file() {
            temp_env::with_vars(
                [("UPDATE_GOLDEN", None), ("TEST_TEMP_RETAIN", Some("1"))],
                || {
                    let temp_dir = test_temp_dir!();
                    temp_dir.used_by(|dir| {
                        let got_err =
                            Golden::new(dir.join("not_existing.txt")).expect_err("this must fail");

                        assert!(Regex::new("Golden file .* not found")
                            .unwrap()
                            .is_match(&got_err.to_string()));
                    });
                },
            );
        }

        #[test]
        fn assert_succeeds_on_correct_golden() {
            temp_env::with_vars(
                [("UPDATE_GOLDEN", None), ("TEST_TEMP_RETAIN", Some("1"))],
                || {
                    let temp_dir = test_temp_dir!();
                    temp_dir.used_by(|dir| {
                        let golden_path = dir.join("golden.txt");
                        std::fs::write(&golden_path, b"The quick fox")
                            .expect("golden file creation failed");

                        let golden = Golden::new(golden_path).unwrap();
                        golden.assert("The quick fox");
                    });
                },
            );
        }

        #[test]
        fn assert_fails_on_different_golden() {
            temp_env::with_vars(
                [("UPDATE_GOLDEN", None), ("TEST_TEMP_RETAIN", Some("1"))],
                || {
                    let temp_dir = test_temp_dir!();
                    temp_dir.used_by(|dir| {
                        let golden_path = dir.join("golden.txt");
                        std::fs::write(&golden_path, b"The quick fox")
                            .expect("golden file creation failed");

                        let golden = Golden::new(golden_path).unwrap();
                        let got_err = std::panic::catch_unwind(|| golden.assert("いろはにほへと"))
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
                },
            );
        }
    }

    mod update_golden_set {
        use super::*;

        #[test]
        fn assert_creates_golden_when_file_not_exists() {
            temp_env::with_vars(
                [
                    ("UPDATE_GOLDEN", Some("1")),
                    ("TEST_TEMP_RETAIN", Some("1")),
                ],
                || {
                    let temp_dir = test_temp_dir!();
                    temp_dir.used_by(|dir| {
                        let golden_path = dir.join("golden_updated.txt");
                        let golden = Golden::new(golden_path.clone())
                            .expect("golden creation should succeed");

                        golden.assert("Veni, vidi, vici\n");

                        assert_str_eq!(read_as_utf8(&golden_path).unwrap(), "Veni, vidi, vici\n");
                    });
                },
            );
        }

        #[test]
        fn assert_updates_golden_when_file_content_different() {
            temp_env::with_vars(
                [
                    ("UPDATE_GOLDEN", Some("1")),
                    ("TEST_TEMP_RETAIN", Some("1")),
                ],
                || {
                    let temp_dir = test_temp_dir!();
                    temp_dir.used_by(|dir| {
                        let golden_path = dir.join("golden_different.txt");
                        std::fs::write(&golden_path, b"numbergirl\n")
                            .expect("golden file creation failed");

                        let golden = Golden::new(golden_path.clone())
                            .expect("golden creation should succeed");

                        golden.assert("Zazen\nBoys\n\n");

                        assert_str_eq!(read_as_utf8(&golden_path).unwrap(), "Zazen\nBoys\n\n");
                    });
                },
            );
        }
    }
}
