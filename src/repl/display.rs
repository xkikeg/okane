//! repl::display contains data & functions for displaying repl data.

use super::*;

use rust_decimal::Decimal;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/// Context information to control the formatting of the transaction.
#[derive(Default)]
pub struct DisplayContext {
    pub precisions: HashMap<String, u8>,
}

impl DisplayContext {
    /// Returns given object reference wrapped with a context for `fmt::Display`.
    pub fn as_display<'a, T>(&'a self, value: &'a T) -> WithContext<'a, T>
    where
        WithContext<'a, T>: fmt::Display,
    {
        WithContext {
            value,
            context: self,
        }
    }
}

/// Object combined with the `DisplayContext`.
pub struct WithContext<'a, T> {
    value: &'a T,
    context: &'a DisplayContext,
}

impl<'a, T> WithContext<'a, T> {
    fn pass_context<U>(&self, other: &'a U) -> WithContext<'a, U> {
        WithContext {
            value: other,
            context: self.context,
        }
    }
}

impl<'a> fmt::Display for WithContext<'a, LedgerEntry> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            LedgerEntry::Txn(txn) => write!(f, "{}", self.pass_context(txn)),
            LedgerEntry::Comment(v) => write!(f, "{}", v),
            LedgerEntry::ApplyTag(v) => v.fmt(f),
            LedgerEntry::EndApplyTag => writeln!(f, "end apply tag"),
            LedgerEntry::Include(v) => v.fmt(f),
            LedgerEntry::Account(v) => v.fmt(f),
            LedgerEntry::Commodity(v) => v.fmt(f),
        }
    }
}

#[derive(Debug)]
struct LineWrapStr<'a> {
    prefix: &'static str,
    content: &'a str,
}

impl<'a> LineWrapStr<'a> {
    fn wrap(prefix: &'static str, content: &'a str) -> Self {
        Self { prefix, content }
    }
}

impl<'a> fmt::Display for LineWrapStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.content.lines() {
            writeln!(f, "{}{}", self.prefix, line)?;
        }
        Ok(())
    }
}

impl fmt::Display for TopLevelComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        LineWrapStr::wrap(";", &self.0).fmt(f)
    }
}

impl fmt::Display for ApplyTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "apply tag {}", self.key)?;
        match &self.value {
            None => writeln!(f),
            Some(v) => writeln!(f, ": {}", v),
        }
    }
}

impl fmt::Display for IncludeFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "include {}", self.0)
    }
}

impl fmt::Display for AccountDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "account {}", self.name)?;
        for detail in &self.details {
            detail.fmt(f)?;
        }
        Ok(())
    }
}
impl fmt::Display for AccountDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountDetail::Comment(v) => LineWrapStr::wrap("    ; ", v).fmt(f),
            AccountDetail::Note(v) => LineWrapStr::wrap("    note ", v).fmt(f),
            AccountDetail::Alias(v) => writeln!(f, "    alias {}", v),
        }
    }
}

impl fmt::Display for CommodityDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "commodity {}", self.name)?;
        for detail in &self.details {
            detail.fmt(f)?;
        }
        Ok(())
    }
}
impl fmt::Display for CommodityDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommodityDetail::Comment(v) => LineWrapStr::wrap("    ; ", v).fmt(f),
            CommodityDetail::Note(v) => LineWrapStr::wrap("    note ", v).fmt(f),
            CommodityDetail::Alias(v) => writeln!(f, "    alias {}", v),
        }
    }
}
impl<'a> fmt::Display for WithContext<'a, Transaction> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xact = self.value;
        write!(f, "{}", xact.date.format("%Y/%m/%d"))?;
        if let Some(edate) = &xact.effective_date {
            write!(f, "={}", edate.format("%Y/%m/%d"))?;
        }
        write!(f, " {}", print_clear_state(xact.clear_state))?;
        if let Some(code) = &xact.code {
            write!(f, "({}) ", code)?;
        }
        writeln!(f, "{}", xact.payee)?;
        for m in &xact.metadata {
            writeln!(f, "    ; {}", m)?;
        }
        for post in &xact.posts {
            write!(
                f,
                "{}",
                WithContext {
                    value: post,
                    context: self.context
                }
            )?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for WithContext<'a, Posting> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let post = self.value;
        let post_clear = print_clear_state(post.clear_state);
        write!(f, "    {}{}", post_clear, post.account)?;
        let account_width =
            UnicodeWidthStr::width_cjk(post.account.as_str()) + UnicodeWidthStr::width(post_clear);
        if let Some(amount) = &post.amount {
            let mut amount_str = String::new();
            let alignment = self
                .pass_context(&amount.amount)
                .fmt_with_alignment(&mut amount_str)?
                .absolute();
            write!(
                f,
                "{:>width$}{}",
                "",
                amount_str.as_str(),
                width = get_column(48, account_width + alignment, 2)
            )?;
            write!(f, "{}", self.pass_context(&amount.lot))?;
            if let Some(exchange) = &amount.cost {
                match exchange {
                    Exchange::Rate(v) => write!(f, " @ {}", self.pass_context(v)),
                    Exchange::Total(v) => write!(f, " @@ {}", self.pass_context(v)),
                }?
            }
        }
        if let Some(balance) = &post.balance {
            let mut balance_str = String::new();
            let alignment = self
                .pass_context(balance)
                .fmt_with_alignment(&mut balance_str)?
                .absolute();
            let trailing = UnicodeWidthStr::width_cjk(balance_str.as_str()) - alignment;
            let balance_padding = if post.amount.is_some() {
                0
            } else {
                get_column(50 + trailing, account_width, 2)
            };
            write!(
                f,
                "{:>width$} {}",
                " =",
                self.pass_context(balance),
                width = balance_padding
            )?;
        }
        writeln!(f)?;
        for m in &post.metadata {
            writeln!(f, "    ; {}", m)?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for WithContext<'a, Lot> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(price) = &self.value.price {
            match price {
                Exchange::Total(e) => write!(f, " {{{{{}}}}}", self.pass_context(e)),
                Exchange::Rate(e) => write!(f, " {{{}}}", self.pass_context(e)),
            }?;
        }
        if let Some(date) = &self.value.date {
            write!(f, " [{}]", date.format("%Y/%m/%d"))?;
        }
        if let Some(note) = &self.value.note {
            write!(f, " ({})", note.as_str())?;
        }
        Ok(())
    }
}

/// Alignment of the expression.
#[derive(Debug, PartialEq, Copy, Clone)]
enum Alignment {
    /// Still alignment wasn't found.
    Partial(usize),
    /// Already alignment was found.
    Complete(usize),
}

impl Alignment {
    fn absolute(self) -> usize {
        match self {
            Alignment::Complete(x) => x,
            Alignment::Partial(x) => x,
        }
    }

    fn plus(self, prefix_length: usize, suffix_length: usize) -> Alignment {
        match self {
            Alignment::Partial(x) => Alignment::Partial(prefix_length + x + suffix_length),
            Alignment::Complete(x) => Alignment::Complete(prefix_length + x),
        }
    }
}

trait DisplayWithAlignment {
    fn fmt_with_alignment<W: fmt::Write>(&self, f: &mut W) -> Result<Alignment, fmt::Error>;
}

impl<'a, T> fmt::Display for WithContext<'a, T>
where
    Self: DisplayWithAlignment,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_alignment(f).map(|_| ())
    }
}

/// Prints expression under the context, and also returns the length until the alignment.
/// Example:
/// - `0` -> returns 1.
/// - `(1 + 1)` -> returns 7.
/// - `2 USD` -> returns 1.
/// - `(1 USD * 2)` -> returns 2.
/// - `(2 * 1 USD)` -> returns 6.
impl<'a> DisplayWithAlignment for WithContext<'a, expr::ValueExpr> {
    fn fmt_with_alignment<W: fmt::Write>(&self, f: &mut W) -> Result<Alignment, fmt::Error> {
        match self.value {
            expr::ValueExpr::Amount(a) => self.pass_context(a).fmt_with_alignment(f),
            expr::ValueExpr::Paren(expr) => {
                write!(f, "(")?;
                let alignment = self.pass_context(expr).fmt_with_alignment(f)?;
                write!(f, ")")?;
                Ok(alignment.plus(1, 1))
            }
        }
    }
}

impl<'a> DisplayWithAlignment for WithContext<'a, expr::Expr> {
    fn fmt_with_alignment<W: fmt::Write>(&self, f: &mut W) -> Result<Alignment, fmt::Error> {
        match self.value {
            expr::Expr::Unary(e) => {
                write!(f, "{}", e.op)?;
                self.pass_context(e.expr.as_ref())
                    .fmt_with_alignment(f)
                    .map(|x| x.plus(1, 0))
            }
            expr::Expr::Binary(e) => {
                let a1 = self.pass_context(e.lhs.as_ref()).fmt_with_alignment(f)?;
                write!(f, " {} ", e.op)?;
                let a2 = self.pass_context(e.rhs.as_ref()).fmt_with_alignment(f)?;
                Ok(match a1.plus(0, 3) {
                    Alignment::Complete(x) => Alignment::Complete(x),
                    Alignment::Partial(x) => a2.plus(x, 0),
                })
            }
            expr::Expr::Value(e) => self.pass_context(e.as_ref()).fmt_with_alignment(f),
        }
    }
}

impl<'a> DisplayWithAlignment for WithContext<'a, Amount> {
    fn fmt_with_alignment<W: fmt::Write>(&self, f: &mut W) -> Result<Alignment, fmt::Error> {
        let amount_str = rescale(self.value, self.context).to_string();
        // TODO: Implement prefix-amount.
        if self.value.commodity.is_empty() {
            write!(f, "{}", amount_str)?;
            return Ok(Alignment::Partial(amount_str.as_str().len()));
        }
        write!(f, "{} {}", amount_str, self.value.commodity)?;
        // Given the amount is only [0-9.], it's ok to count bytes.
        Ok(Alignment::Complete(amount_str.as_str().len()))
    }
}

/// Returns column shift size so that the string will be located at `colsize`.
/// At least `padding` is guaranteed to be spaced.
fn get_column(colsize: usize, left: usize, padding: usize) -> usize {
    if left + padding < colsize {
        colsize - left
    } else {
        padding
    }
}

fn rescale(x: &Amount, context: &DisplayContext) -> Decimal {
    let mut v = x.value;
    v.rescale(std::cmp::max(
        x.value.scale(),
        context.precisions.get(&x.commodity).cloned().unwrap_or(0) as u32,
    ));
    v
}

fn print_clear_state(v: ClearState) -> &'static str {
    match v {
        ClearState::Uncleared => "",
        ClearState::Cleared => "* ",
        ClearState::Pending => "! ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    fn amount<T: Into<Decimal>>(value: T, commodity: &'static str) -> expr::ValueExpr {
        expr::ValueExpr::Amount(Amount {
            commodity: commodity.to_string(),
            value: value.into(),
        })
    }

    fn amount_expr<T: Into<Decimal>>(value: T, commodity: &'static str) -> expr::Expr {
        expr::Expr::Value(Box::new(amount(value, commodity)))
    }

    #[test]
    fn display_ledger_entries_no_txn() {
        let ctx = DisplayContext::default();
        assert_eq!(
            concat!(";this\n", ";is\n", ";a pen pineapple apple pen.\n"),
            format!(
                "{}",
                ctx.as_display(&LedgerEntry::Comment(TopLevelComment(
                    "this\nis\na pen pineapple apple pen.".to_string(),
                )))
            )
        );
        assert_eq!(
            "apply tag foo\n",
            format!(
                "{}",
                ctx.as_display(&LedgerEntry::ApplyTag(ApplyTag {
                    key: "foo".to_string(),
                    value: None
                })),
            )
        );
        assert_eq!(
            "apply tag foo: bar\n",
            format!(
                "{}",
                ctx.as_display(&LedgerEntry::ApplyTag(ApplyTag {
                    key: "foo".to_string(),
                    value: Some("bar".to_string())
                }))
            ),
        );
        assert_eq!(
            "end apply tag\n",
            format!("{}", ctx.as_display(&LedgerEntry::EndApplyTag))
        );
    }

    #[test]
    fn display_txn() {
        let got = format!(
            "{}",
            DisplayContext::default().as_display(&LedgerEntry::Txn(Transaction {
                date: NaiveDate::from_ymd_opt(2022, 12, 23).unwrap(),
                effective_date: None,
                clear_state: ClearState::Uncleared,
                code: None,
                payee: "Example Grocery".to_string(),
                posts: vec![Posting {
                    account: "Assets".to_string(),
                    clear_state: ClearState::Uncleared,
                    amount: Some(PostingAmount {
                        amount: amount(dec!(123.45), "USD"),
                        cost: None,
                        lot: Lot::default(),
                    }),
                    balance: None,
                    metadata: Vec::new(),
                }],
                metadata: Vec::new(),
            }))
        );
        let want = concat!(
            "2022/12/23 Example Grocery\n",
            "    Assets                                    123.45 USD\n",
        );
        assert_eq!(want, got);
    }

    #[test]
    fn posting_non_expr() {
        let all = Posting {
            amount: Some(PostingAmount {
                amount: amount(1, "USD"),
                cost: Some(Exchange::Rate(amount(100, "JPY"))),
                lot: Lot {
                    price: Some(Exchange::Rate(amount(dec!(1.1), "USD"))),
                    date: Some(NaiveDate::from_ymd_opt(2022, 5, 20).unwrap()),
                    note: Some("printable note".to_string()),
                },
            }),
            balance: Some(amount(1, "USD")),
            ..Posting::new("Account".to_string())
        };
        let costbalance = Posting {
            amount: Some(PostingAmount {
                amount: amount(1, "USD"),
                cost: Some(Exchange::Rate(amount(100, "JPY"))),
                lot: Lot::default(),
            }),
            balance: Some(amount(1, "USD")),
            ..Posting::new("Account".to_string())
        };
        let total = Posting {
            amount: Some(PostingAmount {
                amount: amount(1, "USD"),
                cost: Some(Exchange::Total(amount(100, "JPY"))),
                lot: Lot::default(),
            }),
            ..Posting::new("Account".to_string())
        };
        let nocost = Posting {
            amount: Some(PostingAmount {
                amount: amount(1, "USD"),
                cost: None,
                lot: Lot::default(),
            }),
            balance: Some(amount(1, "USD")),
            ..Posting::new("Account".to_string())
        };
        let noamount = Posting {
            amount: None,
            balance: Some(amount(1, "USD")),
            ..Posting::new("Account".to_string())
        };
        let zerobalance = Posting {
            amount: None,
            balance: Some(amount(0, "")),
            ..Posting::new("Account".to_string())
        };

        assert_eq!(
            concat!(
                //       10        20        30        40        50        60        70
                // 34567890123456789012345678901234567890123456789012345678901234567890
                "    Account                                        1 USD {1.1 USD} [2022/05/20] (printable note) @ 100 JPY = 1 USD\n",
                "    Account                                        1 USD @ 100 JPY = 1 USD\n",
                "    Account                                        1 USD @@ 100 JPY\n",
                "    Account                                        1 USD = 1 USD\n",
                "    Account                                              = 1 USD\n",
                // we don't have shared state to determine where = should be aligned
                "    Account                                          = 0\n"
            ),
            format!(
                "{}{}{}{}{}{}",
                DisplayContext::default().as_display(&all),
                DisplayContext::default().as_display(&costbalance),
                DisplayContext::default().as_display(&total),
                DisplayContext::default().as_display(&nocost),
                DisplayContext::default().as_display(&noamount),
                DisplayContext::default().as_display(&zerobalance),
            ),
        );

        let ctx = DisplayContext {
            precisions: hashmap! {"USD".to_string() => 4},
        };
        assert_eq!(
            concat!(
                //       10        20        30        40        50        60        70
                // 34567890123456789012345678901234567890123456789012345678901234567890
                "    Account                                   1.0000 USD {1.1000 USD} [2022/05/20] (printable note) @ 100 JPY = 1.0000 USD\n",
                "    Account                                   1.0000 USD @ 100 JPY = 1.0000 USD\n",
                "    Account                                   1.0000 USD @@ 100 JPY\n",
                "    Account                                   1.0000 USD = 1.0000 USD\n",
                "    Account                                              = 1.0000 USD\n",
                "    Account                                          = 0\n"
            ),
            format!(
                "{}{}{}{}{}{}",
                ctx.as_display(&all),
                ctx.as_display(&costbalance),
                ctx.as_display(&total),
                ctx.as_display(&nocost),
                ctx.as_display(&noamount),
                ctx.as_display(&zerobalance),
            ),
        );
    }

    #[test]
    fn fmt_with_alignment_simple_amount_without_commodity() {
        let mut buffer = String::new();
        let alignment = DisplayContext::default()
            .as_display(&amount(123i8, ""))
            .fmt_with_alignment(&mut buffer)
            .unwrap();
        assert_eq!("123", buffer.as_str());
        assert_eq!(Alignment::Partial(3), alignment);
    }

    #[test]
    fn fmt_with_alignment_simple_amount_with_commodity() {
        let mut buffer = String::new();
        let usd123 = amount(123i8, "USD");
        let alignment = DisplayContext::default()
            .as_display(&usd123)
            .fmt_with_alignment(&mut buffer)
            .unwrap();
        assert_eq!("123 USD", buffer.as_str());
        assert_eq!(Alignment::Complete(3), alignment);

        buffer.clear();
        let alignment = DisplayContext {
            precisions: hashmap! {"USD".to_string() => 2},
        }
        .as_display(&usd123)
        .fmt_with_alignment(&mut buffer)
        .unwrap();
        assert_eq!("123.00 USD", buffer.as_str());
        assert_eq!(Alignment::Complete(6), alignment);
    }

    #[test]
    fn test_fmt_with_alignment_complex_expr() {
        // ((1.20 + 2.67) * 3.1 USD + 5 USD)
        let expr = expr::ValueExpr::Paren(expr::Expr::Binary(expr::BinaryOpExpr {
            lhs: Box::new(expr::Expr::Binary(expr::BinaryOpExpr {
                lhs: Box::new(expr::Expr::Value(Box::new(expr::ValueExpr::Paren(
                    expr::Expr::Binary(expr::BinaryOpExpr {
                        lhs: Box::new(amount_expr(dec!(1.20), "")),
                        op: expr::BinaryOp::Add,
                        rhs: Box::new(amount_expr(dec!(2.67), "")),
                    }),
                )))),
                op: expr::BinaryOp::Mul,
                rhs: Box::new(amount_expr(dec!(3.1), "USD")),
            })),
            op: expr::BinaryOp::Add,
            rhs: Box::new(amount_expr(5i32, "USD")),
        }));
        let mut got = String::new();
        let alignment = DisplayContext::default()
            .as_display(&expr)
            .fmt_with_alignment(&mut got)
            .unwrap();
        assert_eq!("((1.20 + 2.67) * 3.1 USD + 5 USD)", got.as_str());
        assert_eq!(Alignment::Complete(20), alignment);
    }
}
