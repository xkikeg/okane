//! [migemo](https://www.kaoriya.net/software/cmigemo/) integration for the
//! search bars: a long-lived `cmigemo` process that turns a romaji query into
//! a regex matching the Japanese it stands for.
//!
//! The tool speaks a line protocol over a pipe — one query per line in, one
//! pattern per line out — so [`Migemo`] keeps the process alive for the whole
//! session instead of paying the dictionary load (tens of milliseconds) on
//! every keystroke of an incremental search.
//!
//! Only the pipe protocol lives here; what the resulting pattern is matched
//! against is up to the caller (see
//! [`Translator`](super::report::search::Translator)).

use std::cell::RefCell;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Anything that can go wrong talking to the migemo process.
#[derive(thiserror::Error, Debug)]
pub enum MigemoError {
    #[error("--migemo needs a command to run")]
    EmptyCommand,
    #[error("--migemo command is not a valid shell word list (unbalanced quotes?): {0}")]
    UnparsableCommand(String),
    #[error("failed to spawn the migemo command `{command}`")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to talk to the migemo process")]
    Io(#[from] std::io::Error),
    #[error("the migemo process closed its output")]
    Closed,
    #[error("the migemo process returned no pattern for `{0}`")]
    NoPattern(String),
}

impl MigemoError {
    /// Short tag for the search bar, which has one line to say what is wrong
    /// and still has the match count to show.
    pub fn label(&self) -> &'static str {
        match self {
            MigemoError::Closed => "[migemo exited]",
            MigemoError::NoPattern(_) => "[no migemo pattern]",
            MigemoError::Io(_) => "[migemo I/O error]",
            // The startup failures, which never reach a search bar.
            MigemoError::EmptyCommand
            | MigemoError::UnparsableCommand(_)
            | MigemoError::Spawn { .. } => "[migemo failed]",
        }
    }
}

/// The `--migemo` command line: the program plus its arguments, as split out
/// of the single string the flag takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigemoCommand {
    program: String,
    args: Vec<String>,
}

impl MigemoCommand {
    /// Splits `command` the way a shell would, so a dictionary path with
    /// spaces can be quoted: `--migemo='cmigemo -d "/some dir/migemo-dict"'`.
    pub fn parse(command: &str) -> Result<Self, MigemoError> {
        let mut words = shlex::split(command)
            .ok_or_else(|| MigemoError::UnparsableCommand(command.to_owned()))?
            .into_iter();
        let program = words.next().ok_or(MigemoError::EmptyCommand)?;
        Ok(Self {
            program,
            args: words.collect(),
        })
    }
}

impl fmt::Display for MigemoCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.program)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

/// A running migemo process, queried through its stdin/stdout pipe.
///
/// [`Self::query`] takes `&self` so the client can sit behind a shared handle
/// on the UI state: the pipe is the mutable part and it is guarded here.
#[derive(Debug)]
pub struct Migemo {
    child: RefCell<Child>,
    pipe: RefCell<Pipe<ChildStdin, BufReader<ChildStdout>>>,
}

impl Migemo {
    /// Spawns `command` with its stdin and stdout piped. Its stderr is left
    /// alone deliberately: it would land on the alternate screen mid-session,
    /// but a dictionary that fails to load is worth seeing once the TUI exits.
    pub fn spawn(command: MigemoCommand) -> Result<Self, MigemoError> {
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|source| MigemoError::Spawn {
                command: command.to_string(),
                source,
            })?;
        // Both are `Some`: `spawn` succeeded with the pipes requested above.
        let stdin = child.stdin.take().expect("stdin was piped");
        let stdout = child.stdout.take().expect("stdout was piped");
        Ok(Self {
            child: RefCell::new(child),
            pipe: RefCell::new(Pipe::new(stdin, BufReader::new(stdout))),
        })
    }

    /// The regex `input` stands for, as migemo expands it.
    pub fn query(&self, input: &str) -> Result<String, MigemoError> {
        self.pipe.borrow_mut().query(input)
    }
}

impl Drop for Migemo {
    fn drop(&mut self) {
        // The process outlives us otherwise: it is blocked reading a query
        // that will never come. Kill rather than close the pipe and wait, so
        // that a command which is not migemo at all cannot hang the exit.
        let mut child = self.child.borrow_mut();
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// The line protocol itself, over any pair of streams so it can be exercised
/// without a process.
#[derive(Debug)]
struct Pipe<W, R> {
    stdin: W,
    stdout: R,
    /// Reused between queries; the pattern is copied out of it.
    buf: String,
}

impl<W: Write, R: BufRead> Pipe<W, R> {
    fn new(stdin: W, stdout: R) -> Self {
        Self {
            stdin,
            stdout,
            buf: String::new(),
        }
    }

    /// Writes one query and reads back the pattern it produced.
    ///
    /// A newline in `input` would desynchronize the protocol (the tail would
    /// be answered as a second query), so it is dropped: the search bars never
    /// produce one, and a stray one is not worth failing a keystroke over.
    fn query(&mut self, input: &str) -> Result<String, MigemoError> {
        let query: String = input.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        writeln!(self.stdin, "{query}")?;
        self.stdin.flush()?;
        loop {
            self.buf.clear();
            if self.stdout.read_line(&mut self.buf)? == 0 {
                return Err(MigemoError::Closed);
            }
            let line = self.buf.trim_end_matches(['\n', '\r']);
            if is_banner(line) {
                continue;
            }
            let pattern = strip_prompt(line);
            if pattern.is_empty() {
                return Err(MigemoError::NoPattern(query));
            }
            return Ok(pattern.to_owned());
        }
    }
}

/// Whether `line` is one of the notices `cmigemo` prints on startup when it is
/// not run in quiet mode (`-q`), which precede the first answer.
fn is_banner(line: &str) -> bool {
    line.starts_with("migemo_open(") || line.starts_with("clock")
}

/// Strips the prompt `cmigemo` writes ahead of an answer without `-q`. The
/// prompt has no newline of its own, so it arrives on the answer's line:
/// `QUERY: PATTERN: (ケンサク|検索|...)`.
fn strip_prompt(line: &str) -> &str {
    let mut rest = line;
    loop {
        rest = match rest
            .strip_prefix("QUERY: ")
            .or_else(|| rest.strip_prefix("PATTERN: "))
        {
            Some(stripped) => stripped,
            None => return rest,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    use assert_matches::assert_matches;
    use pretty_assertions::assert_eq;

    fn pipe(stdout: &str) -> Pipe<Vec<u8>, Cursor<Vec<u8>>> {
        Pipe::new(Vec::new(), Cursor::new(stdout.as_bytes().to_vec()))
    }

    #[test]
    fn parse_splits_the_command_like_a_shell() {
        assert_eq!(
            MigemoCommand::parse("cmigemo -q -d /usr/share/cmigemo/utf-8/migemo-dict").unwrap(),
            MigemoCommand {
                program: "cmigemo".to_owned(),
                args: vec![
                    "-q".to_owned(),
                    "-d".to_owned(),
                    "/usr/share/cmigemo/utf-8/migemo-dict".to_owned(),
                ],
            }
        );
        assert_eq!(
            MigemoCommand::parse(r#"cmigemo -d "/some dir/migemo-dict""#)
                .unwrap()
                .args,
            vec!["-d".to_owned(), "/some dir/migemo-dict".to_owned()]
        );
    }

    #[test]
    fn parse_rejects_nothing_to_run() {
        assert_matches!(MigemoCommand::parse(""), Err(MigemoError::EmptyCommand));
        assert_matches!(MigemoCommand::parse("   "), Err(MigemoError::EmptyCommand));
        assert_matches!(
            MigemoCommand::parse("cmigemo -d \"unbalanced"),
            Err(MigemoError::UnparsableCommand(_))
        );
    }

    #[test]
    fn display_round_trips_into_parse() {
        let command = MigemoCommand::parse("cmigemo -q -d dict").unwrap();
        assert_eq!(command.to_string(), "cmigemo -q -d dict");
        assert_eq!(MigemoCommand::parse(&command.to_string()).unwrap(), command);
    }

    #[test]
    fn query_writes_a_line_and_reads_the_pattern() {
        let mut pipe = pipe("(ケンサク|検索|kensaku)\n(ギンコウ|銀行|ginkou)\n");
        assert_eq!(pipe.query("kensaku").unwrap(), "(ケンサク|検索|kensaku)");
        assert_eq!(pipe.query("ginkou").unwrap(), "(ギンコウ|銀行|ginkou)");
        assert_eq!(pipe.stdin, b"kensaku\nginkou\n");
    }

    /// Without `-q`, cmigemo greets with two notices and prefixes every answer
    /// with its (newline-less) prompt. Both are the user's choice of command,
    /// so both are understood rather than refused.
    #[test]
    fn query_skips_the_banner_and_the_prompt() {
        let mut pipe = pipe(concat!(
            "migemo_open(\"/usr/share/cmigemo/utf-8/migemo-dict\")=0x55d0\n",
            "clock()=0.052280\n",
            "QUERY: PATTERN: (ケンサク|検索|kensaku)\n",
            "QUERY: PATTERN: (ギンコウ|銀行|ginkou)\n",
        ));
        assert_eq!(pipe.query("kensaku").unwrap(), "(ケンサク|検索|kensaku)");
        assert_eq!(pipe.query("ginkou").unwrap(), "(ギンコウ|銀行|ginkou)");
    }

    #[test]
    fn query_strips_a_trailing_carriage_return() {
        let mut pipe = pipe("(ケンサク|検索|kensaku)\r\n");
        assert_eq!(pipe.query("kensaku").unwrap(), "(ケンサク|検索|kensaku)");
    }

    /// A newline would be answered as two queries and leave an extra pattern
    /// in the pipe for the next keystroke to read.
    #[test]
    fn query_never_writes_more_than_one_line() {
        let mut pipe = pipe("(ケンサク|検索|kensaku)\n");
        assert_eq!(pipe.query("kensa\nku").unwrap(), "(ケンサク|検索|kensaku)");
        assert_eq!(pipe.stdin, b"kensaku\n");
    }

    #[test]
    fn query_reports_a_dead_process() {
        let mut pipe = pipe("");
        assert_matches!(pipe.query("kensaku"), Err(MigemoError::Closed));
    }

    /// An empty pattern would match every account rather than none, which is
    /// the opposite of what an unanswered query means.
    #[test]
    fn query_reports_an_empty_answer() {
        let mut pipe = pipe("QUERY: \n");
        assert_matches!(pipe.query("kensaku"), Err(MigemoError::NoPattern(q)) if q == "kensaku");
    }
    /// Round trip against a real migemo, exercising the pipe protocol as an
    /// actual tool speaks it — including the startup banner and the prompt of
    /// a command run without `-q`. Skipped unless a command is given, since
    /// neither the tool nor a dictionary is something CI has:
    ///
    /// ```shell
    /// OKANE_TEST_MIGEMO='cmigemo -d /usr/share/cmigemo/utf-8/migemo-dict' cargo test
    /// ```
    #[test]
    fn real_migemo_expands_a_romaji_query() {
        let Ok(command) = std::env::var("OKANE_TEST_MIGEMO") else {
            eprintln!("skipped: set OKANE_TEST_MIGEMO to a migemo command to run this");
            return;
        };
        let migemo = Migemo::spawn(MigemoCommand::parse(&command).unwrap()).unwrap();
        // The same process answers every query of a session, so check that the
        // second one is not the first one's leftovers.
        for (query, account) in [("ginkou", "資産:銀行"), ("shisan", "資産")] {
            let pattern = migemo.query(query).unwrap();
            let re = regex::RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .unwrap_or_else(|err| panic!("{query} expanded to `{pattern}`: {err}"));
            assert!(re.is_match(account), "{query} expanded to `{pattern}`");
        }
    }
}
