//! [migemo](https://www.kaoriya.net/software/cmigemo/) integration for the
//! search bars: a long-lived `cmigemo` process that turns a romaji query into
//! a regex matching the Japanese it stands for.
//!
//! The tool speaks a line protocol over a pipe — one query per line in, one
//! pattern per line out — so [`Migemo`] keeps the process alive for the whole
//! session instead of paying the dictionary load (tens of milliseconds) on
//! every keystroke of an incremental search.
//!
//! [`MigemoDict`] is the in-process alternative: rustmigemo over its compact
//! dictionary, with no process at all.
//!
//! Only the expansion lives here; what the resulting pattern is matched
//! against is up to the caller (see
//! [`Translator`](super::report::search::Translator)).

use std::cell::RefCell;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::panic::{self, UnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use rustmigemo::migemo::compact_dictionary::CompactDictionary;
use rustmigemo::migemo::regex_generator::RegexOperator;

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
    #[error("failed to read the migemo dictionary {}", path.display())]
    ReadDictionary {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "{} is not a rustmigemo compact dictionary (cmigemo's migemo-dict goes to --migemo instead)",
        path.display()
    )]
    InvalidDictionary { path: PathBuf },
    #[error("the migemo dictionary failed to expand `{0}`")]
    DictionaryQuery(String),
}

impl MigemoError {
    /// Short tag for the search bar, which has one line to say what is wrong
    /// and still has the match count to show.
    pub fn label(&self) -> &'static str {
        match self {
            MigemoError::Closed => "[migemo exited]",
            MigemoError::NoPattern(_) => "[no migemo pattern]",
            MigemoError::Io(_) => "[migemo I/O error]",
            // The startup failures, which never reach a search bar, and a
            // dictionary that loaded but cannot answer.
            MigemoError::EmptyCommand
            | MigemoError::UnparsableCommand(_)
            | MigemoError::Spawn { .. }
            | MigemoError::ReadDictionary { .. }
            | MigemoError::InvalidDictionary { .. }
            | MigemoError::DictionaryQuery(_) => "[migemo failed]",
        }
    }
}

/// Where the search bars get migemo from: a command to spawn (`--migemo`), or
/// a dictionary to run rustmigemo over in-process (`--migemo-dict`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigemoSource {
    Command(MigemoCommand),
    Dict(PathBuf),
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

/// Migemo run in-process by [rustmigemo](https://github.com/oguna/rustmigemo)
/// over its compact dictionary, so there is no process to spawn, block on, or
/// fall out of step with: a query is a plain function call.
///
/// The dictionary is the user's to download (`migemo-compact-dict`, from
/// oguna/migemo-compact-dict-latest): it is GPL-licensed, so it is not
/// bundled. It is not the `migemo-dict` cmigemo reads.
pub struct MigemoDict {
    /// Boxed: the tries' bookkeeping is hundreds of bytes inline, which would
    /// set the size of every `Translator`.
    dict: Box<CompactDictionary>,
}

impl fmt::Debug for MigemoDict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The dictionary is megabytes of trie, and has no `Debug` anyway.
        f.debug_struct("MigemoDict").finish_non_exhaustive()
    }
}

impl MigemoDict {
    /// Reads the compact dictionary at `path`.
    pub fn load(path: &Path) -> Result<Self, MigemoError> {
        let bytes = std::fs::read(path).map_err(|source| MigemoError::ReadDictionary {
            path: path.to_owned(),
            source,
        })?;
        Self::from_bytes(&bytes).ok_or_else(|| MigemoError::InvalidDictionary {
            path: path.to_owned(),
        })
    }

    /// Parses a compact dictionary, or `None` when `bytes` is not one.
    ///
    /// The format has no header to check, and rustmigemo trusts every length
    /// it reads: a wrong file (cmigemo's text dictionary, a truncated
    /// download) has it allocate whatever its first bytes say — gigabytes,
    /// which aborts rather than panics — or unwrap a read past the end. So the
    /// layout is walked first ([`fits_compact_layout`]), and a panic left
    /// after that is caught.
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if !fits_compact_layout(bytes) {
            return None;
        }
        let bytes = bytes.to_vec();
        let dict = Box::new(quietly(|| CompactDictionary::new(&bytes))?);
        Some(Self { dict })
    }

    /// The regex `input` stands for, as migemo expands it.
    pub fn query(&self, input: &str) -> Result<String, MigemoError> {
        let pattern = quietly(|| {
            rustmigemo::migemo::query::query(input.to_owned(), &self.dict, &RegexOperator::Default)
        })
        // A dictionary that parsed can still hold indices that point nowhere.
        .ok_or_else(|| MigemoError::DictionaryQuery(input.to_owned()))?;
        if pattern.is_empty() {
            // Only for a blank `input`: an empty regex would match every row.
            return Err(MigemoError::NoPattern(input.to_owned()));
        }
        Ok(pattern)
    }
}

/// Whether `bytes` is exactly the arrays a compact dictionary is made of, each
/// a big-endian `u32` length then its elements, so that no length rustmigemo
/// reads can run past the end of the file:
///
/// - the key trie: edges of 1 byte, then its bit vector in `u64` words;
/// - the value trie: edges of 2 bytes, then its bit vector;
/// - the mapping bit vector;
/// - the mapping, of `u32`s.
///
/// A bit vector's length counts bits, stored as whole `u64` words.
fn fits_compact_layout(bytes: &[u8]) -> bool {
    /// Skips one array of `len` elements of `size` bytes, or of `len` bits
    /// when `size` is 0. `None` once it would leave `bytes`.
    fn skip_array(bytes: &[u8], pos: usize, size: usize) -> Option<usize> {
        let len_bytes = bytes.get(pos..pos.checked_add(4)?)?;
        let len = u32::from_be_bytes(len_bytes.try_into().ok()?) as usize;
        let body = if size == 0 {
            len.div_ceil(64).checked_mul(8)?
        } else {
            len.checked_mul(size)?
        };
        let end = (pos + 4).checked_add(body)?;
        (end <= bytes.len()).then_some(end)
    }
    const BITS: usize = 0;
    [1, BITS, 2, BITS, BITS, 4]
        .into_iter()
        .try_fold(0, |pos, size| skip_array(bytes, pos, size))
        == Some(bytes.len())
}

/// Runs `f`, turning a panic into `None` without printing it: it would land on
/// the TUI's alternate screen otherwise.
fn quietly<T>(f: impl FnOnce() -> T + UnwindSafe) -> Option<T> {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(hook);
    result.ok()
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
pub(crate) mod tests {
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

    /// A compact dictionary with just enough words to tell a romaji query
    /// expanded from one that did not, built the way the published one is.
    fn tiny_dict_bytes() -> Vec<u8> {
        let words = [("ぎんこう", "銀行"), ("しさん", "資産")]
            .into_iter()
            .map(|(reading, word)| (reading.to_owned(), vec![word.to_owned()]))
            .collect();
        rustmigemo::migemo::compact_dictionary_builder::build(words)
    }

    pub(crate) fn tiny_dict() -> MigemoDict {
        MigemoDict::from_bytes(&tiny_dict_bytes()).expect("the builder's output loads")
    }

    fn matches(pattern: &str, text: &str) -> bool {
        regex::RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap_or_else(|err| panic!("`{pattern}` is not a regex: {err}"))
            .is_match(text)
    }

    #[test]
    fn dict_expands_romaji_into_the_words_it_reads_as() {
        let dict = tiny_dict();
        let pattern = dict.query("ginkou").unwrap();
        assert!(matches(&pattern, "資産:銀行"), "{pattern}");
        // The romaji itself stays one of the alternatives.
        assert!(matches(&pattern, "Assets:Ginkou"), "{pattern}");
        assert!(!matches(&pattern, "資産:現金"), "{pattern}");
        // A prefix already reaches the word, as incremental search needs.
        assert!(matches(&dict.query("gin").unwrap(), "銀行"));
    }

    /// The pattern feeds the `regex` crate, so what migemo treats as text must
    /// come out escaped.
    #[test]
    fn dict_escapes_regex_metacharacters() {
        let pattern = tiny_dict().query("a.b").unwrap();
        assert!(matches(&pattern, "a.b"), "{pattern}");
        assert!(!matches(&pattern, "axb"), "{pattern}");
    }

    #[test]
    fn dict_has_no_pattern_for_a_blank_query() {
        assert_matches!(tiny_dict().query(" "), Err(MigemoError::NoPattern(_)));
    }

    /// cmigemo's text dictionary is the likely mix-up, and rustmigemo panics
    /// on it rather than erroring.
    #[test]
    fn load_rejects_what_is_not_a_compact_dictionary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("migemo-dict");
        std::fs::write(&path, "ぎんこう\t銀行\n").unwrap();
        assert_matches!(
            MigemoDict::load(&path),
            Err(MigemoError::InvalidDictionary { path: p }) if p == path
        );
        assert_matches!(
            MigemoDict::load(&dir.path().join("missing")),
            Err(MigemoError::ReadDictionary { .. })
        );
    }

    /// rustmigemo allocates whatever a length says before reading it, so
    /// these must be refused up front: run through it, they abort the process
    /// (out of memory) or panic.
    #[test]
    fn from_bytes_refuses_lengths_past_the_end() {
        let bytes = tiny_dict_bytes();
        assert!(fits_compact_layout(&bytes));
        // Truncated anywhere, including inside a length.
        for len in [0, 3, 4, bytes.len() / 2, bytes.len() - 1] {
            assert!(
                MigemoDict::from_bytes(&bytes[..len]).is_none(),
                "cut at {len}"
            );
        }
        // Trailing bytes mean the file is something else that happens to fit.
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(MigemoDict::from_bytes(&longer).is_none());
        // A first length of 4G edges, as cmigemo's text dictionary reads.
        let mut huge = bytes;
        huge[..4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(MigemoDict::from_bytes(&huge).is_none());
    }

    /// The real dictionary, which CI does not have:
    ///
    /// ```shell
    /// OKANE_TEST_MIGEMO_DICT=path/to/migemo-compact-dict cargo test
    /// ```
    #[test]
    fn real_dict_expands_a_romaji_query() {
        let Ok(path) = std::env::var("OKANE_TEST_MIGEMO_DICT") else {
            eprintln!("skipped: set OKANE_TEST_MIGEMO_DICT to a migemo-compact-dict to run this");
            return;
        };
        let dict = MigemoDict::load(Path::new(&path)).unwrap();
        for (query, account) in [("ginkou", "資産:銀行"), ("shisan", "資産")] {
            let pattern = dict.query(query).unwrap();
            assert!(
                matches(&pattern, account),
                "{query} expanded to `{pattern}`"
            );
        }
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
