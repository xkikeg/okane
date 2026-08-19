//! format functionalities of Ledger format files.

use crate::{
    load::{self, LoadError, Loader},
    parse::{ParseError, ParseOptions, parse_ledger},
    syntax::{
        self,
        display::{DisplayContext, DisplayContextBuilder},
    },
};

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::path::Path;

/// Error occured during Format.
#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse the file")]
    Parse(#[from] ParseError),
    #[error("failed to walk the ledger files")]
    Load(#[from] LoadError),
}

/// Options to control format functionalities.
///
/// This formats one file at a time, with the given [`DisplayContext`].
/// Use [`format_recursively`] to format a whole tree of files instead,
/// which derives the context out of the tree itself.
#[derive(Debug, Default)]
pub struct FormatOptions {
    context: DisplayContext,
}

impl FormatOptions {
    /// Create a default FormatOptions instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets how the amount of each commodity is rendered.
    ///
    /// Without this, every amount is printed the way it was written, as the
    /// commodity settings can't be known from one file alone. Usually the
    /// context comes from a [`DisplayContextBuilder`] fed with every entry of
    /// the ledger, which is what [`format_recursively`] does.
    pub fn with_display_context(self, context: DisplayContext) -> Self {
        Self { context }
    }

    /// Formats the given `input`, and returns the formatted content.
    pub fn format_str(&self, input: &str) -> Result<String, ParseError> {
        let mut ret = String::with_capacity(input.len());
        for parsed in parse_ledger(&ParseOptions::default(), input) {
            let (_, entry): (_, syntax::plain::LedgerEntry) = parsed?;
            // Writing into a String never fails.
            write!(ret, "{}", self.context.as_display(&entry)).expect("write! into String failed");
        }
        Ok(ret)
    }

    /// Formats given `Read` instance and write it back to `Write`.
    pub fn format<R, W>(&self, r: &mut R, w: &mut W) -> Result<(), FormatError>
    where
        R: Read,
        W: Write,
    {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        w.write_all(self.format_str(&buf)?.as_bytes())?;
        Ok(())
    }
}

/// One file of the ledger, formatted: what is on disk, and what it should be.
///
/// This is a pure diff, carrying no decision about what to do with it. Which is
/// [`Emitter`]'s job: writing the file back, printing the difference, counting
/// the files to be formatted, ... whatever the caller is after.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormattedFile<'a> {
    path: &'a Path,
    original: &'a str,
    formatted: &'a str,
}

impl<'a> FormattedFile<'a> {
    /// Creates a new instance out of the file content and its formatted form.
    pub fn new(path: &'a Path, original: &'a str, formatted: &'a str) -> Self {
        Self {
            path,
            original,
            formatted,
        }
    }

    /// Returns the path of the file, as the [`Loader`] resolved it.
    pub fn path(&self) -> &'a Path {
        self.path
    }

    /// Returns the content currently on the disk.
    pub fn original(&self) -> &'a str {
        self.original
    }

    /// Returns the content the file should have.
    pub fn formatted(&self) -> &'a str {
        self.formatted
    }

    /// Returns `true` if the file is already formatted, i.e. there is nothing to apply.
    pub fn is_formatted(&self) -> bool {
        self.original == self.formatted
    }
}

/// Applies whatever action the caller has in mind on every formatted file.
///
/// [`format_recursively`] reports *every* file it walked, formatted or not, so
/// an emitter interested in the changes alone has to skip the ones with
/// [`FormattedFile::is_formatted`].
pub trait Emitter {
    /// Error the action may fail with.
    ///
    /// It must be able to carry the formatter's own errors as well, so that the
    /// caller has a single error type to deal with.
    type Error: From<FormatError>;

    /// Applies the action on the given file.
    fn emit(&mut self, file: FormattedFile<'_>) -> Result<(), Self::Error>;
}

/// Formats the file the `loader` points at and every file it `include`s,
/// reporting each of them to the `emitter` in load order.
///
/// The tree is walked twice, as how an amount is printed is a property of its
/// commodity, and the commodity may well be declared in another file than the
/// one being printed. So nothing can be printed before every file has been
/// looked at.
///
/// 1. [`Loader::scan_files`] parses every file to follow the `include`
///    directives, and the entries it reports on the way feed a
///    [`DisplayContextBuilder`].
/// 2. Every file found is parsed again, and printed with the resulting
///    [`DisplayContext`].
pub fn format_recursively<F, E>(loader: &Loader<F>, emitter: &mut E) -> Result<(), E::Error>
where
    F: load::FileSystem,
    E: Emitter,
{
    let mut builder = DisplayContextBuilder::new();
    // `?` converts the errors into `E::Error`, as `Emitter` requires.
    let paths = loader
        .scan_files(|_, entry| builder.observe(entry))
        .map_err(FormatError::from)?;
    let options = FormatOptions::new().with_display_context(builder.build());
    for path in paths {
        let original = loader
            .filesystem()
            .file_content_utf8(&path)
            .map_err(|err| FormatError::from(LoadError::IO(err, path.clone())))?;
        let formatted = options
            .format_str(&original)
            .map_err(|err| FormatError::from(LoadError::Parse(err, path.clone())))?;
        emitter.emit(FormattedFile::new(&path, &original, &formatted))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    fn format_str(input: &str) -> String {
        format_with(&FormatOptions::new(), input)
    }

    fn format_with(options: &FormatOptions, input: &str) -> String {
        let mut output = Vec::new();
        options
            .format(&mut input.as_bytes(), &mut output)
            .expect("format() should succeeds");
        String::from_utf8(output).expect("output should be valid UTF-8")
    }

    /// Formats the input with the context derived from the input itself,
    /// the way [`format_recursively`] does over a whole tree of files.
    fn format_with_inferred_context(input: &str) -> String {
        let mut builder = DisplayContextBuilder::new();
        for parsed in parse_ledger::<syntax::plain::Ident>(&ParseOptions::default(), input) {
            let (_, entry) = parsed.expect("input should be parsed");
            builder.observe(&entry);
        }
        format_with(
            &FormatOptions::new().with_display_context(builder.build()),
            input,
        )
    }

    #[test]
    fn format_keeps_comment_attached_to_the_next_entry() {
        let input = indoc! {"
            ; Explains the transaction below.
            2021/03/12 Opening Balance
                Assets:Bank                                          = 1000 CHF
                Equity
            ; Explains the transaction above.

            ; Standalone comment, separated by blank lines.

            2021/03/13 Grocery
                Expenses:Grocery                              10 CHF
                Assets:Bank
        "};

        assert_eq!(input, format_str(input));
    }

    #[test]
    fn format_collapses_consecutive_blank_lines() {
        let input = indoc! {"
            include a.ledger



            include b.ledger
        "};
        let want = indoc! {"
            include a.ledger

            include b.ledger
        "};

        assert_eq!(want, format_str(input));
    }

    #[test]
    fn format_is_idempotent() {
        let input = indoc! {"
            ; Top
            #comment

            account  Foo\t
             ; a comment about Foo
             alias Bar

            commodity USD
             ; a comment about USD
             alias $

            apply    tag   foo
            end  apply   tag

            2021/05/14 !(#txn-1) My Grocery
                ; a comment about the transaction
                Expenses:Grocery\t10 CHF
                Assets:Bank  -10 CHF
        "};

        let once = format_str(input);
        assert_eq!(once, format_str(&once), "formatting must be idempotent");
        // The comment prefixes must not accumulate spaces across runs.
        assert!(
            once.contains("    ; a comment about Foo\n"),
            "got:\n{}",
            once
        );
        assert!(
            once.contains("    ; a comment about USD\n"),
            "got:\n{}",
            once
        );
    }

    #[test]
    fn format_applies_the_display_context() {
        let input = indoc! {"
            2021/05/14 My Grocery
                Expenses:Grocery                       1234.5 CHF
                Assets:Bank                           -1234.5 CHF
        "};
        let mut builder = DisplayContextBuilder::new();
        builder.declare(
            "CHF",
            syntax::display::CommodityDisplayOption {
                format: Some(pretty_decimal::Format::Comma3Dot),
                min_scale: Some(2),
            },
        );
        let want = indoc! {"
            2021/05/14 My Grocery
                Expenses:Grocery                        1,234.50 CHF
                Assets:Bank                            -1,234.50 CHF
        "};

        assert_eq!(
            want,
            format_with(
                &FormatOptions::new().with_display_context(builder.build()),
                input
            )
        );
    }

    #[test]
    fn format_with_inferred_context_is_a_fixed_point() {
        let input = indoc! {"
            commodity JPY
                format 1,000 JPY

            2021/05/14 My Grocery
                Expenses:Grocery       1234 JPY
                Expenses:Household     12.30 CHF
                Expenses:Commissions   1,000 CHF
                Assets:Broker          2 SPINX {1.23456 CHF} @ 1.23456 CHF
                Assets:Complex         (-10 * 2.1 CHF) = 0
                Assets:Bank            -5678.9 CHF
        "};

        let once = format_with_inferred_context(input);
        assert_eq!(
            once,
            format_with_inferred_context(&once),
            "formatting must be a fixed point"
        );
        // The declared format applies, even though the amount has no comma.
        assert!(once.contains("1,234 JPY"), "got:\n{}", once);
        // The widest CHF amount decides the scale, and the comma spreads.
        assert!(once.contains("1,000.00 CHF"), "got:\n{}", once);
        assert!(once.contains("-5,678.90 CHF"), "got:\n{}", once);
        // The cost keeps its own precision, and the bare number is untouched.
        assert!(
            once.contains("{1.23456 CHF} @ 1.23456 CHF"),
            "got:\n{}",
            once
        );
        assert!(once.contains("= 0\n"), "got:\n{}", once);
    }

    #[test]
    fn format_succeeds_transaction_without_lot_price() {
        let input = indoc! {"
            ; Top
            ; level
            #comment
            %can
            |have several prefixes.

            ; second
            ; round

            account  Foo\t
             alias Bar\t
               note これは何でしょうか
              alias Baz

            commodity  USD\t
             \talias 米ドル\t
             \talias $\t

            apply    tag   foo

            apply tag key: value
            apply tag key:: 10 USD

            end  apply   tag
            ; key:: 10 USD

            end apply tag
            end apply tag

            include        path/to/other.ledger

            2021/03/12 Opening Balance  ; initial balance
             Assets:Bank     = 1000 CHF
             Equity

            2021/05/14 !(#txn-1) My Grocery
                Expenses:Grocery\t10 CHF
                Expenses:Commissions    1 USD   @ 0.98 CHF ; Payee: My Card
                ; My card took commission
                ; :financial:経済:
                Assets:Bank  -20 CHF=1CHF
                Expenses:Household  = 0
                Assets:Complex  (-10 * 2.1 $) @ (1 $ + 1 $) = 2.5 $
                Assets:Broker  -2 SPINX (bought before Xmas) {100 USD} [2010/12/23] @ 10000 USD
                Liabilities:Comma      5,678.00 CHF @ 1,000,000 JPYRIN = -123,456.12 CHF
        "};
        // TODO: 1. guess commodity width if not available.
        // TOOD: 2. remove trailing space on non-commodity value.
        let want = indoc! {"
            ; Top
            ; level
            ;comment
            ;can
            ;have several prefixes.

            ; second
            ; round

            account Foo
                alias Bar
                note これは何でしょうか
                alias Baz

            commodity USD
                alias 米ドル
                alias $

            apply tag foo

            apply tag key: value
            apply tag key:: 10 USD

            end apply tag
            ; key:: 10 USD

            end apply tag
            end apply tag

            include path/to/other.ledger

            2021/03/12 Opening Balance
                ; initial balance
                Assets:Bank                                          = 1000 CHF
                Equity

            2021/05/14 ! (#txn-1) My Grocery
                Expenses:Grocery                              10 CHF
                Expenses:Commissions                           1 USD @ 0.98 CHF
                ; Payee: My Card
                ; My card took commission
                ; :financial:経済:
                Assets:Bank                                  -20 CHF = 1 CHF
                Expenses:Household                               = 0
                Assets:Complex                        (-10 * 2.1 $) @ (1 $ + 1 $) = 2.5 $
                Assets:Broker                                 -2 SPINX {100 USD} [2010/12/23] (bought before Xmas) @ 10000 USD
                Liabilities:Comma                       5,678.00 CHF @ 1,000,000 JPYRIN = -123,456.12 CHF
        "};
        let mut output = Vec::new();
        let mut r = input.as_bytes();

        FormatOptions::new()
            .format(&mut r, &mut output)
            .expect("format() should succeeds");
        let got = std::str::from_utf8(&output).expect("output should be valid UTF-8");
        assert_eq!(want, got);
    }
}

#[cfg(test)]
mod format_recursively_tests {
    use super::*;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::load::FakeFileSystem;

    /// Error of the test emitters.
    #[derive(thiserror::Error, Debug)]
    enum TestError {
        #[error(transparent)]
        Format(#[from] FormatError),
        #[error("emit refused {0}")]
        Refused(PathBuf),
    }

    /// [`Emitter`] keeping every reported file, applying nothing.
    #[derive(Default)]
    struct Recorder(Vec<(PathBuf, String, bool)>);

    impl Emitter for Recorder {
        type Error = TestError;

        fn emit(&mut self, file: FormattedFile<'_>) -> Result<(), TestError> {
            self.0.push((
                file.path().to_owned(),
                file.formatted().to_string(),
                file.is_formatted(),
            ));
            Ok(())
        }
    }

    fn loader_of(files: HashMap<PathBuf, Vec<u8>>) -> Loader<FakeFileSystem> {
        Loader::new(
            PathBuf::from("path/to/root.ledger"),
            FakeFileSystem::from(files),
        )
    }

    #[test]
    fn format_recursively_reports_every_file_with_the_context_of_the_whole_tree() {
        let loader = loader_of(hashmap! {
            PathBuf::from("path/to/root.ledger") => indoc! {"
                include child.ledger

                2024/01/01 Lunch
                    Expenses:Grocery       1234 JPY
                    Assets:Bank
            "}.as_bytes().to_vec(),
            // Already formatted, and the only place JPY is declared.
            PathBuf::from("path/to/child.ledger") => indoc! {"
                commodity JPY
                    format 1,000 JPY
            "}.as_bytes().to_vec(),
        });
        let mut recorder = Recorder::default();

        format_recursively(&loader, &mut recorder).expect("format_recursively must succeed");

        assert_eq!(
            vec![
                (
                    PathBuf::from("path/to/root.ledger"),
                    indoc! {"
                        include child.ledger

                        2024/01/01 Lunch
                            Expenses:Grocery                           1,234 JPY
                            Assets:Bank
                    "}
                    .to_string(),
                    // The commodity is declared in the included file, yet it
                    // applies to the amount written in the root file.
                    false,
                ),
                (
                    PathBuf::from("path/to/child.ledger"),
                    "commodity JPY\n    format 1,000 JPY\n".to_string(),
                    true,
                ),
            ],
            recorder.0
        );
    }

    #[test]
    fn format_recursively_returns_the_emitter_error() {
        let loader = loader_of(hashmap! {
            PathBuf::from("path/to/root.ledger") => "account Expenses:Grocery\n".as_bytes().to_vec(),
        });
        struct Refuse;
        impl Emitter for Refuse {
            type Error = TestError;

            fn emit(&mut self, file: FormattedFile<'_>) -> Result<(), TestError> {
                Err(TestError::Refused(file.path().to_owned()))
            }
        }

        let got = format_recursively(&loader, &mut Refuse).expect_err("emit must fail");

        assert!(
            matches!(&got, TestError::Refused(p) if p == Path::new("path/to/root.ledger")),
            "unexpected error: {:?}",
            got
        );
    }

    #[test]
    fn format_recursively_reports_the_unreadable_file() {
        let loader = loader_of(hashmap! {
            PathBuf::from("path/to/other.ledger") => "".as_bytes().to_vec(),
        });

        let got = format_recursively(&loader, &mut Recorder::default())
            .expect_err("format_recursively must fail");

        assert!(
            matches!(
                &got,
                TestError::Format(FormatError::Load(LoadError::IO(_, p)))
                    if p == Path::new("path/to/root.ledger")
            ),
            "unexpected error: {:?}",
            got
        );
    }
}
