use crate::repl::{
    display::DisplayContext,
    parser::{parse_ledger, ParseLedgerError},
    LedgerEntry,
};

use std::io::{Read, Write};

#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("nom failed to parse the file")]
    Parse(#[from] ParseLedgerError),
}

/// Converts given string into formatted string.
pub fn format<R, W>(r: &mut R, w: &mut W) -> Result<(), FormatError>
where
    R: Read,
    W: Write,
{
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let txns = parse_ledger(&buf)?;
    let ctx = DisplayContext::default();
    for txn in txns {
        match txn {
            LedgerEntry::Txn(txn) => writeln!(w, "{}", txn.display(&ctx))?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn format_succeeds_transaction_without_lot_price() {
        let input = indoc! {"
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
        "};
        // TODO: 1. guess commodity width if not available.
        // TOOD: 2. remove trailing space on non-commodity value.
        let want = indoc! {"
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

        "};
        let mut output = Vec::new();
        let mut r = input.as_bytes();

        format(&mut r, &mut output).expect("format() should succeeds");
        let got = std::str::from_utf8(&output).expect("output should be valid UTF-8");
        assert_eq!(want, got);
    }
}
