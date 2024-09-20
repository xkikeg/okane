//! format functionalities of Ledger format files.

use crate::{
    parse::{parse_ledger, ParseError},
    syntax::{self, display::DisplayContext},
};

use std::io::{Read, Write};

/// Error occured during Format.
#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse the file")]
    Parse(#[from] Box<ParseError>),
    // TODO: Remove this once supported.
    #[error("recursive format isn't supported yet")]
    UnsupportedRecursiveFormat,
}

/// Options to control format functionalities.
#[derive(Debug, Default)]
pub struct FormatOptions {
    recursive: bool,
}

impl FormatOptions {
    /// Create a default FormatOptions instance.
    /// All options are initially set to `false`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `recursive` option to given value.
    /// If `recursive` is set to `true`, it'll try to follow `include` directive.
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Formats given `Read` instance and write it back to `Write`.
    pub fn format<R, W>(&self, r: &mut R, w: &mut W) -> Result<(), FormatError>
    where
        R: Read,
        W: Write,
    {
        // TODO: Implement recursive formatting.
        if self.recursive {
            return Err(FormatError::UnsupportedRecursiveFormat);
        }
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        // TODO: Grab DisplayContext externally, or from LedgerEntry.
        let ctx = DisplayContext::default();
        for parsed in parse_ledger(&buf) {
            let (_, entry): (_, syntax::plain::LedgerEntry) = parsed.map_err(Box::new)?;
            writeln!(w, "{}", ctx.as_display(&entry))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

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
