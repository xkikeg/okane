use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use okane_core::{format, load, syntax::display::DisplayContextBuilder};

/// Converts given string into formatted string.
pub fn format<R, W>(
    options: &format::FormatOptions,
    r: &mut R,
    w: &mut W,
) -> Result<(), format::FormatError>
where
    R: Read,
    W: Write,
{
    options.format(r, w)
}

/// A file whose formatted content differs from what is on disk.
#[derive(Debug)]
pub struct Reformatted {
    pub path: PathBuf,
    original: String,
    formatted: String,
}

/// Formats `source` and every file reachable through its `include` directives,
/// returning only the files whose content would change.
///
/// The tree is walked twice: how an amount is printed depends on its commodity,
/// which may well be declared in another file, so nothing can be printed before
/// every file has been looked at.
pub fn collect_reformatted(source: &Path) -> anyhow::Result<Vec<Reformatted>> {
    let mut builder = DisplayContextBuilder::new();
    let paths = load::new_loader(source.to_owned())
        .scan_files(|_, entry| builder.observe(entry))
        .with_context(|| format!("failed to load {}", source.display()))?;
    let options = format::FormatOptions::new().with_display_context(builder.build());
    let mut ret = Vec::new();
    for path in paths {
        let original = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut formatted = Vec::new();
        format(&options, &mut original.as_bytes(), &mut formatted)
            .with_context(|| format!("failed to format {}", path.display()))?;
        let formatted = String::from_utf8(formatted)
            .with_context(|| format!("formatted {} is not valid UTF-8", path.display()))?;
        if formatted != original {
            ret.push(Reformatted {
                path,
                original,
                formatted,
            });
        }
    }
    Ok(ret)
}

/// Writes the formatted content back to the file.
///
/// The content goes through a temporary file in the same directory first, so an
/// interrupted run cannot leave a half-written ledger behind.
pub fn write_back(target: &Reformatted) -> anyhow::Result<()> {
    let dir = target
        .path
        .parent()
        .with_context(|| format!("{} does not have a parent directory", target.path.display()))?;
    let mut temp = tempfile::NamedTempFile::new_in(dir)
        .with_context(|| format!("failed to create a temporary file in {}", dir.display()))?;
    temp.write_all(target.formatted.as_bytes())
        .with_context(|| format!("failed to write the formatted {}", target.path.display()))?;
    // Temporary files are created with restrictive permissions, so the original
    // mode has to be carried over explicitly.
    let permissions = std::fs::metadata(&target.path)
        .with_context(|| format!("failed to stat {}", target.path.display()))?
        .permissions();
    temp.as_file()
        .set_permissions(permissions)
        .with_context(|| format!("failed to set permissions of {}", target.path.display()))?;
    temp.persist(&target.path)
        .with_context(|| format!("failed to replace {}", target.path.display()))?;
    log::info!("formatted {}", display_path(&target.path));
    Ok(())
}

/// Renders the path the way it is shown to the user.
///
/// [`load::Loader::list_files`] canonicalizes, so paths are absolute; shortening
/// them against the current directory keeps the output readable, and keeps it
/// independent of where the ledger tree happens to live.
fn display_path(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Writes the unified diff between the current and the formatted content.
pub fn write_diff<W>(w: &mut W, target: &Reformatted) -> anyhow::Result<()>
where
    W: Write,
{
    let display = display_path(&target.path);
    write!(
        w,
        "{}",
        similar::TextDiff::from_lines(&target.original, &target.formatted)
            .unified_diff()
            .header(&display, &display)
    )
    .with_context(|| format!("failed to write the diff of {}", target.path.display()))
}
