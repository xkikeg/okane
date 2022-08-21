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

/// Object combined with the `DisplayContext`.
pub struct WithContext<'a, T> {
    value: &'a T,
    context: &'a DisplayContext,
}

impl Transaction {
    pub fn display<'a>(&'a self, context: &'a DisplayContext) -> WithContext<'a, Transaction> {
        WithContext {
            value: self,
            context,
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
            if let expr::ValueExpr::Amount(amount) = &amount.amount {
                let amount_str = rescale(amount, self.context).to_string();
                write!(
                    f,
                    "{:>width$}{} {}",
                    "",
                    amount_str.as_str(),
                    amount.commodity,
                    width = get_column(
                        48,
                        account_width + UnicodeWidthStr::width(amount_str.as_str()),
                        2
                    )
                )?;
            }
            if let Some(exchange) = &amount.cost {
                match exchange {
                    Exchange::Rate(expr::ValueExpr::Amount(v)) => {
                        write!(f, " @ {} {}", v.value, v.commodity)
                    }
                    Exchange::Total(expr::ValueExpr::Amount(v)) => {
                        write!(
                            f,
                            " @@ {} {}",
                            display::rescale(v, self.context),
                            v.commodity
                        )
                    }
                    _ => todo!("non-literal value expression isn't supported yet"),
                }?
            }
        }
        if let Some(expr::ValueExpr::Amount(balance)) = &post.balance {
            let balance_padding = if post.amount.is_some() {
                0
            } else {
                get_column(
                    51 + UnicodeWidthStr::width_cjk(balance.commodity.as_str()),
                    account_width,
                    3,
                )
            };
            write!(
                f,
                "{:>width$} {}",
                " =",
                rescale(balance, self.context),
                width = balance_padding
            )?;
            if !balance.commodity.is_empty() {
                write!(f, " {}", balance.commodity)?;
            }
        }
        writeln!(f)?;
        for m in &post.metadata {
            writeln!(f, "    ; {}", m)?;
        }
        Ok(())
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
