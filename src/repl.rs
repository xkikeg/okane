//! repl represents Ledger file format representation.
//! Note the structure is quite similar to data module,
//! however, repl is for textual representation while
//! data is more for understanding.

pub mod parser;

use crate::data;
pub use crate::data::{Amount, ClearState, Exchange, ExchangedAmount};

use std::collections::HashMap;
use std::fmt;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use unicode_width::UnicodeWidthStr;

/// Top-level entry of the LedgerFile.
#[derive(Debug, PartialEq)]
pub enum LedgerEntry {
    Txn(Transaction),
}

/// Represents a transaction where the money transfered across the accounts.
#[derive(Debug, PartialEq)]
pub struct Transaction {
    /// Date when the transaction issued.
    pub date: NaiveDate,
    /// Date when the transaction got effective, optional.
    pub effective_date: Option<NaiveDate>,
    /// Indiacates clearing state of the entire transaction.
    pub clear_state: ClearState,
    /// Transaction code (not necessarily unique).
    pub code: Option<String>,
    /// Label of the transaction, often the opposite party of the transaction.
    pub payee: String,
    /// Postings of the transaction, could be empty.
    pub posts: Vec<Post>,
}

impl Transaction {
    /// Constructs minimal transaction.
    pub fn new(date: NaiveDate, payee: String) -> Transaction {
        Transaction {
            date,
            effective_date: None,
            clear_state: ClearState::Uncleared,
            code: None,
            payee,
            posts: Vec::new(),
        }
    }
}

impl From<&data::Transaction> for Transaction {
    fn from(orig: &data::Transaction) -> Transaction {
        Transaction {
            date: orig.date,
            effective_date: orig.effective_date,
            clear_state: orig.clear_state,
            code: orig.code.clone(),
            payee: orig.payee.clone(),
            posts: orig.posts.iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Debug, PartialEq)]
/// Post is a posting in a transaction, and
/// it represents a particular account increase / decrease.
pub struct Post {
    /// Account of the post target.
    pub account: String,
    /// Posting specific ClearState.
    pub clear_state: ClearState,
    /// Amount of the posting.
    pub amount: Option<ExchangedAmount>,
    /// Balance after the transaction of the specified account.
    pub balance: Option<Amount>,
    /// Metadata information such as comment or tag.
    pub metadata: Vec<Metadata>,
}

impl Post {
    pub fn new(account: String) -> Post {
        Post {
            account,
            clear_state: ClearState::default(),
            amount: None,
            balance: None,
            metadata: Vec::new(),
        }
    }
}

impl From<&data::Post> for Post {
    fn from(orig: &data::Post) -> Post {
        let metadata = orig
            .payee
            .iter()
            .map(|v| Metadata::KeyValueTag {
                key: "Payee".to_string(),
                value: v.clone(),
            })
            .collect();
        Post {
            account: orig.account.clone(),
            clear_state: orig.clear_state,
            amount: orig.amount.clone(),
            balance: orig.balance.clone(),
            metadata,
        }
    }
}

/// Metadata represents meta information associated with transactions / posts.
#[derive(Debug, PartialEq)]
pub enum Metadata {
    /// Comment, which covers just one line.
    Comment(String),
    /// Tags of word, in a format :tag1:tag2:tag3:
    WordTags(Vec<String>),
    /// Key-value paired tag.
    KeyValueTag { key: String, value: String },
}

/// Parses number including comma, returns the decimal.
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

/// Context information to control the formatting of the transaction.
pub struct DisplayContext {
    pub precisions: HashMap<String, u8>,
}

impl DisplayContext {
    pub fn empty() -> DisplayContext {
        DisplayContext {
            precisions: HashMap::new(),
        }
    }
}

/// Transaction combined with the transaction.
pub struct TransactionWithContext<'a> {
    pub transaction: &'a Transaction,
    pub context: &'a DisplayContext,
}

fn rescale(x: &Amount, context: &DisplayContext) -> Decimal {
    let mut v = x.value;
    v.rescale(std::cmp::max(
        x.value.scale(),
        context.precisions.get(&x.commodity).cloned().unwrap_or(0) as u32,
    ));
    v
}

fn get_column(colsize: usize, left: usize, padding: usize) -> usize {
    if left + padding < colsize {
        colsize - left
    } else {
        padding
    }
}

impl<'a> fmt::Display for TransactionWithContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xact = self.transaction;
        write!(f, "{}", xact.date.format("%Y/%m/%d"))?;
        if let Some(edate) = &xact.effective_date {
            write!(f, "={}", edate.format("%Y/%m/%d"))?;
        }
        write!(f, " {}", print_clear_state(xact.clear_state))?;
        if let Some(code) = &xact.code {
            write!(f, "({}) ", code)?;
        }
        writeln!(f, "{}", xact.payee)?;
        for post in &xact.posts {
            let post_clear = print_clear_state(post.clear_state);
            write!(f, "    {}{}", post_clear, post.account)?;
            let account_width = UnicodeWidthStr::width_cjk(post.account.as_str())
                + UnicodeWidthStr::width(post_clear);
            if let Some(amount) = &post.amount {
                let amount_str = rescale(&amount.amount, self.context).to_string();
                write!(
                    f,
                    "{:>width$}{} {}",
                    "",
                    rescale(&amount.amount, self.context),
                    amount.amount.commodity,
                    width = get_column(
                        48,
                        account_width + UnicodeWidthStr::width(amount_str.as_str()),
                        2
                    )
                )?;
                if let Some(exchange) = &amount.exchange {
                    match exchange {
                        Exchange::Rate(v) => write!(f, " @ {} {}", v.value, v.commodity),
                        Exchange::Total(v) => {
                            write!(f, " @@ {} {}", rescale(v, self.context), v.commodity)
                        }
                    }?
                }
            }
            if let Some(balance) = &post.balance {
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
                    "{:>width$} {} {}",
                    " =",
                    rescale(balance, self.context),
                    balance.commodity,
                    width = balance_padding
                )?;
            }
            writeln!(f)?;
            for m in &post.metadata {
                write!(f, "    ; ")?;
                match m {
                    Metadata::Comment(c) => write!(f, "{}", c)?,
                    Metadata::WordTags(tags) => {
                        for (i, tag) in tags.iter().enumerate() {
                            if i == 0 {
                                write!(f, ":")?;
                            }
                            write!(f, "{}:", tag)?;
                        }
                    }
                    Metadata::KeyValueTag { key, value } => write!(f, "{}: {}", key, value)?,
                }
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

impl Transaction {
    pub fn display<'a>(&'a self, context: &'a DisplayContext) -> TransactionWithContext<'a> {
        TransactionWithContext {
            transaction: self,
            context,
        }
    }
}
