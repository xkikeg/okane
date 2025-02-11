use std::path::{Path, PathBuf};

use pretty_assertions::assert_str_eq;

/// Returns content of testdata directory, as UTF-8.
pub fn read_as_utf8(filename: &Path) -> std::io::Result<String> {
    // Needs to replace CRLF to LF for Windows, as all files may have CRLF
    // depending on the git config core.autocrlf.
    std::fs::read_to_string(filename).map(|s| s.replace("\r\n", "\n"))
}

/// Golden file, which maintains the expected content and assert over that.
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
                    log::warn!("Golden not found: pass UPDATE_GOLDEN=1");
                    Err(e)
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
