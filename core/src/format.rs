//! format functionalities of Ledger format files.

use crate::{
    parse::{ParseError, ParseOptions, parse_ledger},
    syntax::{self, display::DisplayContext},
};

use std::io::{Read, Write};

/// Error occured during Format.
#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse the file")]
    Parse(#[from] ParseError),
}

/// Options to control format functionalities.
///
/// This formats one file at a time. To format a whole tree of files, walk
/// [`load::Loader::scan_files()`][crate::load::Loader::scan_files], feeding the
/// entries into a [`DisplayContextBuilder`][builder], and format each file with
/// the resulting context.
///
/// [builder]: crate::syntax::display::DisplayContextBuilder
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
    /// context comes from a
    /// [`DisplayContextBuilder`][crate::syntax::display::DisplayContextBuilder]
    /// fed with every entry of the ledger.
    pub fn with_display_context(self, context: DisplayContext) -> Self {
        Self { context }
    }

    /// Formats given `Read` instance and write it back to `Write`.
    pub fn format<R, W>(&self, r: &mut R, w: &mut W) -> Result<(), FormatError>
    where
        R: Read,
        W: Write,
    {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        for parsed in parse_ledger(&ParseOptions::default(), &buf) {
            let (_, entry): (_, syntax::plain::LedgerEntry) = parsed?;
            write!(w, "{}", self.context.as_display(&entry))?;
        }
        Ok(())
    }
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
    /// the way the formatter does over a whole tree of files.
    fn format_with_inferred_context(input: &str) -> String {
        let mut builder = syntax::display::DisplayContextBuilder::new();
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
        let mut builder = syntax::display::DisplayContextBuilder::new();
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
