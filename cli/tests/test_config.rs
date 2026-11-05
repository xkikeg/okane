//! Tests of the config files, `okane.toml` and the user config.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;

pub mod testing;

#[ctor::ctor(unsafe)]
fn init() {
    env_logger::init();
}

/// Temporary home directory, holding the config files and the Ledger.
struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new() -> Self {
        let sandbox = Sandbox {
            dir: tempfile::tempdir().unwrap(),
        };
        std::fs::create_dir_all(sandbox.path("books/2026")).unwrap();
        std::fs::copy(
            testing::TESTDATA_DIR.join("report/multi_commodity.ledger"),
            sandbox.path("books/main.ledger"),
        )
        .unwrap();
        sandbox
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    /// okane run from `cwd`, with the sandbox as the home directory.
    fn okane(&self, cwd: &str) -> assert_cmd::Command {
        let xdg = self.path("xdg");
        let mut cmd = assert_cmd::Command::new(&*testing::BIN_PATH);
        cmd.current_dir(self.path(cwd))
            .env("HOME", self.dir.path())
            .env("USERPROFILE", self.dir.path())
            .env("XDG_CONFIG_HOME", &xdg)
            .env("APPDATA", &xdg)
            .env_remove("OKANE_CONFIG");
        cmd
    }

    /// Path of the user config.
    fn user_config(&self) -> &'static str {
        "xdg/okane/config.toml"
    }
}

/// TOML literal string of `path`, which keeps Windows backslashes as is.
fn toml_path(path: &Path) -> String {
    format!("'{}'", path.display())
}

fn stdout_of(cmd: &mut assert_cmd::Command) -> String {
    let output = cmd.assert().success().get_output().clone();
    String::from_utf8(output.stdout).unwrap()
}

/// Output of okane without any config, for the expected values.
fn without_config(args: &[&OsStr]) -> String {
    stdout_of(testing::okane().args(args))
}

#[test]
fn project_config_gives_source_and_report_flags() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "books/okane.toml",
        "source = 'main.ledger'\n[report]\nexchange = 'USD'\nhistorical = true\n",
    );
    let ledger = sandbox.path("books/main.ledger");

    let got = stdout_of(sandbox.okane("books/2026").arg("balance"));

    let want = without_config(&[
        "balance".as_ref(),
        "-f".as_ref(),
        ledger.as_os_str(),
        "-X".as_ref(),
        "USD".as_ref(),
        "--historical".as_ref(),
    ]);
    assert_eq!(got, want);
}

#[test]
fn command_line_wins_over_config() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "books/okane.toml",
        "source = 'main.ledger'\n[report]\nexchange = 'NO-SUCH'\nhistorical = true\n",
    );
    let ledger = sandbox.path("books/main.ledger");

    let got = stdout_of(
        sandbox
            .okane("books")
            .args(["balance", "-X", "USD", "--no-historical"]),
    );

    let want = without_config(&[
        "balance".as_ref(),
        "-f".as_ref(),
        ledger.as_os_str(),
        "-X".as_ref(),
        "USD".as_ref(),
    ]);
    assert_eq!(got, want);
}

#[test]
fn command_table_overrides_report_table() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "books/okane.toml",
        "source = 'main.ledger'\n[report]\nexchange = 'NO-SUCH'\n[register]\nexchange = 'USD'\nhistorical = true\nsort = 'date'\n",
    );
    let ledger = sandbox.path("books/main.ledger");

    let got = stdout_of(sandbox.okane("books").arg("register"));

    let want = without_config(&[
        "register".as_ref(),
        "-f".as_ref(),
        ledger.as_os_str(),
        "-X".as_ref(),
        "USD".as_ref(),
        "--historical".as_ref(),
        "--sort".as_ref(),
        "date".as_ref(),
    ]);
    assert_eq!(got, want);
}

#[test]
fn positionals_are_account_patterns_when_config_gives_source() {
    let sandbox = Sandbox::new();
    sandbox.write("books/okane.toml", "source = 'main.ledger'\n");
    let ledger = sandbox.path("books/main.ledger");

    let output = sandbox
        .okane("books")
        .args(["balance", "Assets"])
        .assert()
        .success()
        .get_output()
        .clone();

    let want = without_config(&[
        "balance".as_ref(),
        "-f".as_ref(),
        ledger.as_os_str(),
        "Assets".as_ref(),
    ]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), want);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[rstest::rstest]
#[case::balance("balance")]
#[case::register("register")]
fn positional_ledger_file_still_works_with_a_warning(#[case] command: &str) {
    let ledger = testing::TESTDATA_DIR.join("report/multi_commodity.ledger");

    let output = testing::okane()
        .args([command.as_ref(), ledger.as_os_str(), "Assets".as_ref()])
        .assert()
        .success()
        .get_output()
        .clone();

    let want = without_config(&[
        command.as_ref(),
        "-f".as_ref(),
        ledger.as_os_str(),
        "Assets".as_ref(),
    ]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), want);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("passing the Ledger file as the first argument is deprecated"),
        "{stderr}"
    );
}

#[rstest::rstest]
#[case::accounts("accounts")]
#[case::tags("tags")]
fn source_from_user_config(#[case] command: &str) {
    let sandbox = Sandbox::new();
    let ledger = sandbox.path("books/main.ledger");
    sandbox.write(
        sandbox.user_config(),
        &format!("source = {}\n", toml_path(&ledger)),
    );

    let got = stdout_of(sandbox.okane("books").arg(command));

    let want = without_config(&[command.as_ref(), ledger.as_os_str()]);
    assert_eq!(got, want);
}

#[test]
fn project_config_wins_over_user_config() {
    let sandbox = Sandbox::new();
    sandbox.write(
        sandbox.user_config(),
        "source = 'no-such.ledger'\n[report]\nexchange = 'USD'\n",
    );
    sandbox.write("books/okane.toml", "source = 'main.ledger'\n");
    let ledger = sandbox.path("books/main.ledger");

    let got = stdout_of(sandbox.okane("books").arg("balance"));

    let want = without_config(&[
        "balance".as_ref(),
        "-f".as_ref(),
        ledger.as_os_str(),
        "-X".as_ref(),
        "USD".as_ref(),
    ]);
    assert_eq!(got, want);
}

#[test]
fn okane_config_replaces_discovery() {
    let sandbox = Sandbox::new();
    sandbox.write("books/okane.toml", "broken");
    sandbox.write(
        "elsewhere/okane-test.toml",
        "source = '../books/main.ledger'\n",
    );
    let ledger = sandbox.path("books/main.ledger");

    let got = stdout_of(
        sandbox
            .okane("books")
            .env("OKANE_CONFIG", sandbox.path("elsewhere/okane-test.toml"))
            .arg("accounts"),
    );

    let want = without_config(&["accounts".as_ref(), ledger.as_os_str()]);
    assert_eq!(got, want);
}

#[test]
fn no_config_skips_broken_config() {
    let sandbox = Sandbox::new();
    sandbox.write("books/okane.toml", "broken");

    let got = stdout_of(
        sandbox
            .okane("books")
            .args(["--no-config", "accounts", "main.ledger"]),
    );

    let ledger = sandbox.path("books/main.ledger");
    let want = without_config(&["accounts".as_ref(), ledger.as_os_str()]);
    assert_eq!(got, want);
}

#[test]
fn invalid_config_is_an_error_naming_the_file_and_key() {
    let sandbox = Sandbox::new();
    sandbox.write("books/okane.toml", "[report]\nprice_db = 'x'\n");

    let output = sandbox
        .okane("books")
        .args(["accounts", "main.ledger"])
        .assert()
        .failure()
        .code(2)
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("okane.toml"), "{stderr}");
    assert!(stderr.contains("price_db"), "{stderr}");
}

#[test]
fn missing_ledger_file_is_an_error() {
    let output = testing::okane()
        .arg("balance")
        .assert()
        .failure()
        .code(2)
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("no Ledger file given"), "{stderr}");
}

#[test]
fn import_rules_from_config() {
    let sandbox = Sandbox::new();
    let rules = testing::TESTDATA_DIR.join("import/test_config.yml");
    sandbox.write(
        "books/okane.toml",
        &format!("[import]\nrules = {}\n", toml_path(&rules)),
    );
    let input = testing::TESTDATA_DIR.join("import/csv_delimiter.csv");

    let got = stdout_of(
        sandbox
            .okane("books")
            .args(["import".as_ref(), input.as_os_str()]),
    );

    let want = without_config(&[
        "import".as_ref(),
        "--rules".as_ref(),
        rules.as_os_str(),
        input.as_os_str(),
    ]);
    assert_eq!(got, want);
}
