use rust_decimal::Decimal;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub date: chrono::NaiveDate,
    pub clear_state: ClearState,
    pub code: Option<String>,
    pub payee: String,
    pub posts: Vec<Post>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClearState {
    /// No specific meaning.
    Uncleared,
    /// Useful to show the transaction / post is confirmed.
    Cleared,
    /// Useful to show the transaction / post is still pending.
    Pending,
}

impl Default for ClearState {
    fn default() -> ClearState {
        ClearState::Uncleared
    }
}

#[derive(Debug, PartialEq)]
pub struct Post {
    pub account: String,
    pub clear_state: ClearState,
    pub amount: Amount,
    pub balance: Option<Amount>,
}

#[derive(Debug, PartialEq)]
pub struct Amount {
    pub value: Decimal,
    pub commodity: String,
}

pub fn parse_comma_decimal(x: &str) -> Result<Decimal, rust_decimal::Error> {
    x.replace(',', "").parse()
}

fn print_clear_state(v: ClearState) -> &'static str {
    match v {
        ClearState::Uncleared => "",
        ClearState::Cleared => "* ",
        ClearState::Pending => "! ",
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.date.format("%Y/%m/%d"),
            print_clear_state(self.clear_state)
        )?;
        if let Some(code) = &self.code {
            write!(f, "({}) ", code)?;
        }
        writeln!(f, "{}", self.payee)?;
        for post in &self.posts {
            let post_clear = print_clear_state(post.clear_state);
            write!(
                f,
                "    {}{}{:>width$} {}",
                post_clear,
                post.account,
                post.amount.value,
                post.amount.commodity,
                width = 48 - post.account.len() - post_clear.len()
            )?;
            if let Some(balance) = &post.balance {
                write!(f, " = {} {}", balance.value, balance.commodity)?;
            }
            writeln!(f, "")?;
        }
        return Ok(());
    }
}
