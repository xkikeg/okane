use std::io::Write;
use std::path::{Path, PathBuf};

pub mod testing;

#[ctor::ctor(unsafe)]
fn init() {
    env_logger::init();
}

/// Every file of the fixture tree, relative to its root.
const FIXTURE_FILES: [&str; 3] = ["main.ledger", "sub/child.ledger", "sub/commodities.ledger"];

/// Copies the fixture tree into a fresh temporary directory, so that formatting
/// in place never touches the files in the repository.
fn fixture_dir() -> tempfile::TempDir {
    let src = testing::TESTDATA_DIR.join("format");
    let dir = tempfile::tempdir().expect("failed to create a temporary directory");
    for relative in FIXTURE_FILES {
        let dest = dir.path().join(relative);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::copy(src.join(relative), &dest)
            .unwrap_or_else(|e| panic!("failed to copy {} : {}", relative, e));
    }
    dir
}

fn okane(dir: &Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::new(&*testing::BIN_PATH);
    // Run from the fixture directory and pass relative paths, so the diff
    // headers do not contain the temporary directory name.
    cmd.current_dir(dir);
    cmd
}

fn golden(name: &str) -> okane_golden::Golden {
    let path = testing::TESTDATA_DIR
        .join("format")
        .join("golden")
        .join(name);
    okane_golden::Golden::new(path).unwrap()
}

fn read(dir: &Path, relative: &str) -> String {
    std::fs::read_to_string(dir.join(relative))
        .unwrap_or_else(|e| panic!("failed to read {} : {}", relative, e))
}

/// Golden file name for a fixture file's formatted content.
fn formatted_golden_name(relative: &str) -> String {
    format!("{}.golden.format.txt", relative.replace('/', "_"))
}

#[test]
fn format_check_reports_diff_and_fails() {
    let dir = fixture_dir();
    let before: Vec<String> = FIXTURE_FILES.iter().map(|f| read(dir.path(), f)).collect();

    let result = okane(dir.path())
        .args(["format", "--check", "main.ledger"])
        .assert()
        .code(1);

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    // Windows renders the included path with a backslash.
    golden("check.golden.diff.txt").assert(&stdout.replace('\\', "/"));

    let after: Vec<String> = FIXTURE_FILES.iter().map(|f| read(dir.path(), f)).collect();
    assert_eq!(before, after, "--check must not modify any file");
}

#[test]
fn format_rewrites_every_included_file_in_place() {
    let dir = fixture_dir();

    let result = okane(dir.path())
        .args(["format", "main.ledger"])
        .assert()
        .success();
    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(
        "",
        std::str::from_utf8(&output.stdout).unwrap(),
        "in-place format must not write to stdout"
    );

    for relative in FIXTURE_FILES {
        golden(&formatted_golden_name(relative)).assert(&read(dir.path(), relative));
    }
}

#[test]
fn format_is_idempotent_and_check_passes_afterwards() {
    let dir = fixture_dir();
    okane(dir.path())
        .args(["format", "main.ledger"])
        .assert()
        .success();
    let once: Vec<String> = FIXTURE_FILES.iter().map(|f| read(dir.path(), f)).collect();

    okane(dir.path())
        .args(["format", "main.ledger"])
        .assert()
        .success();
    let twice: Vec<String> = FIXTURE_FILES.iter().map(|f| read(dir.path(), f)).collect();
    assert_eq!(once, twice, "formatting twice must be a no-op");

    okane(dir.path())
        .args(["format", "--check", "main.ledger"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn format_accepts_fmt_alias() {
    let dir = fixture_dir();
    okane(dir.path())
        .args(["fmt", "main.ledger"])
        .assert()
        .success();
    golden(&formatted_golden_name("main.ledger")).assert(&read(dir.path(), "main.ledger"));
}

#[test]
fn primitive_format_prints_one_file_without_touching_it() {
    let dir = fixture_dir();
    let before = read(dir.path(), "main.ledger");

    let result = okane(dir.path())
        .args(["primitive", "format", "main.ledger"])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    // Not the golden of the recursive format: the commodities declared in the
    // included files are not picked up here, and the amounts stay as written.
    golden("main.ledger.golden.primitive-format.txt").assert(stdout);

    assert_eq!(
        before,
        read(dir.path(), "main.ledger"),
        "primitive format must not modify the file"
    );
    let child: PathBuf = dir.path().join("sub/child.ledger");
    assert_eq!(
        std::fs::read_to_string(&child).unwrap(),
        read(&testing::TESTDATA_DIR.join("format"), "sub/child.ledger"),
        "primitive format must not follow include"
    );
}

#[test]
fn primitive_format_applies_commodity_format() {
    let dir = fixture_dir();

    let result = okane(dir.path())
        .args([
            "primitive",
            "format",
            "--commodity-format",
            "JPY=1,000",
            "--commodity-format",
            "OKANE=1.0000",
            "main.ledger",
        ])
        .assert()
        .success();

    let output = result.get_output();
    std::io::stderr().write_all(&output.stderr).unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("1,234 JPY"), "got:\n{}", stdout);
    assert!(stdout.contains("2.0000 OKANE"), "got:\n{}", stdout);
    // Not given, so it stays the way it is written.
    assert!(stdout.contains("12.30 CHF"), "got:\n{}", stdout);
}

#[test]
fn primitive_format_rejects_invalid_commodity_format() {
    let dir = fixture_dir();

    let result = okane(dir.path())
        .args([
            "primitive",
            "format",
            "--commodity-format",
            "CHF=oops",
            "main.ledger",
        ])
        .assert()
        .failure();

    let stderr = std::str::from_utf8(&result.get_output().stderr).unwrap();
    assert!(stderr.contains("invalid sample amount"), "got:\n{}", stderr);
}
