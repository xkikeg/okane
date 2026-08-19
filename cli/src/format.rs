//! Emitters deciding what happens to the result of `okane format`.
//!
//! [`okane_core::format::format_recursively`] only computes what every file
//! should look like; whether that is written back or shown as a diff is up to
//! the [`Emitter`] given to it.

use std::io::Write;
use std::path::Path;

use anyhow::Context as _;
use okane_core::format::{Emitter, FormattedFile};

/// [`Emitter`] rewriting each file in place.
///
/// The content goes through a temporary file in the same directory first, so an
/// interrupted run cannot leave a half-written ledger behind.
#[derive(Debug, Default)]
pub struct WriteBack;

impl Emitter for WriteBack {
    type Error = anyhow::Error;

    fn emit(&mut self, file: FormattedFile<'_>) -> anyhow::Result<()> {
        if file.is_formatted() {
            return Ok(());
        }
        let path = file.path();
        let dir = path
            .parent()
            .with_context(|| format!("{} does not have a parent directory", path.display()))?;
        let mut temp = tempfile::NamedTempFile::new_in(dir)
            .with_context(|| format!("failed to create a temporary file in {}", dir.display()))?;
        temp.write_all(file.formatted().as_bytes())
            .with_context(|| format!("failed to write the formatted {}", path.display()))?;
        // Temporary files are created with restrictive permissions, so the
        // original mode has to be carried over explicitly.
        let permissions = std::fs::metadata(path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .permissions();
        temp.as_file()
            .set_permissions(permissions)
            .with_context(|| format!("failed to set permissions of {}", path.display()))?;
        temp.persist(path)
            .with_context(|| format!("failed to replace {}", path.display()))?;
        log::info!("formatted {}", display_path(path));
        Ok(())
    }
}

/// [`Emitter`] printing what [`WriteBack`] would do, without touching any file.
///
/// Remembers whether anything would change, so that the caller can report it
/// through the exit code.
#[derive(Debug)]
pub struct WriteDiff<W> {
    out: W,
    has_diff: bool,
}

impl<W: Write> WriteDiff<W> {
    /// Creates an instance writing the diffs into `out`.
    pub fn new(out: W) -> Self {
        Self {
            out,
            has_diff: false,
        }
    }

    /// Returns `true` if any file reported so far was not formatted.
    pub fn has_diff(&self) -> bool {
        self.has_diff
    }
}

impl<W: Write> Emitter for WriteDiff<W> {
    type Error = anyhow::Error;

    fn emit(&mut self, file: FormattedFile<'_>) -> anyhow::Result<()> {
        if file.is_formatted() {
            return Ok(());
        }
        self.has_diff = true;
        let display = display_path(file.path());
        write!(
            self.out,
            "{}",
            similar::TextDiff::from_lines(file.original(), file.formatted())
                .unified_diff()
                .header(&display, &display)
        )
        .with_context(|| format!("failed to write the diff of {}", file.path().display()))
    }
}

/// Renders the path the way it is shown to the user.
///
/// [`okane_core::load::Loader`] canonicalizes, so paths are absolute;
/// shortening them against the current directory keeps the output readable, and
/// keeps it independent of where the ledger tree happens to live.
fn display_path(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn diff_of(original: &str, formatted: &str) -> String {
        let mut emitter = WriteDiff::new(Vec::new());
        emitter
            .emit(FormattedFile::new(
                Path::new("main.ledger"),
                original,
                formatted,
            ))
            .expect("emit must succeed");
        assert_eq!(original != formatted, emitter.has_diff());
        String::from_utf8(emitter.out).expect("diff must be valid UTF-8")
    }

    #[test]
    fn write_diff_renders_the_unified_diff() {
        assert_eq!(
            indoc::indoc! {"
                --- main.ledger
                +++ main.ledger
                @@ -1,2 +1,2 @@
                 account Assets:Bank
                - alias  Bank
                +    alias Bank
            "},
            diff_of(
                "account Assets:Bank\n alias  Bank\n",
                "account Assets:Bank\n    alias Bank\n"
            )
        );
    }

    #[test]
    fn write_diff_stays_quiet_on_the_formatted_file() {
        let content = "account Assets:Bank\n";
        assert_eq!("", diff_of(content, content));
    }

    #[test]
    fn write_back_replaces_the_file_content_only_when_needed() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("main.ledger");
        std::fs::write(&path, "account Assets:Bank\n alias  Bank\n")?;

        WriteBack.emit(FormattedFile::new(
            &path,
            "account Assets:Bank\n alias  Bank\n",
            "account Assets:Bank\n    alias Bank\n",
        ))?;
        assert_eq!(
            "account Assets:Bank\n    alias Bank\n",
            std::fs::read_to_string(&path)?
        );

        // An already formatted file is left alone, mtime included.
        let before = std::fs::metadata(&path)?.modified()?;
        WriteBack.emit(FormattedFile::new(
            &path,
            "account Assets:Bank\n    alias Bank\n",
            "account Assets:Bank\n    alias Bank\n",
        ))?;
        assert_eq!(before, std::fs::metadata(&path)?.modified()?);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn write_back_keeps_the_file_permissions() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("main.ledger");
        std::fs::write(&path, "account Assets:Bank\n")?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))?;

        WriteBack.emit(FormattedFile::new(
            &path,
            "account Assets:Bank\n",
            "account Assets:Bank\n; formatted\n",
        ))?;

        assert_eq!(
            0o644,
            std::fs::metadata(&path)?.permissions().mode() & 0o777
        );
        Ok(())
    }
}
