//! repl represents Ledger file format representation.
//! Note the structure is quite similar to data module,
//! however, repl is for textual representation while
//! data is more for understanding.

pub mod display;
pub mod expr;
pub mod parser;

use crate::data;
pub use crate::data::{Amount, ClearState};

use std::fmt;

use chrono::NaiveDate;

/// Top-level entry of the LedgerFile.
#[derive(Debug, PartialEq, Eq)]
pub enum LedgerEntry {
    Txn(Transaction),
}

/// Represents a transaction where the money transfered across the accounts.
#[derive(Debug, PartialEq, Eq)]
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
    pub posts: Vec<Posting>,
    /// Transaction level metadata.
    metadata: Vec<Metadata>,
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
            metadata: Vec::new(),
            posts: Vec::new(),
        }
    }
}

impl From<data::Transaction> for Transaction {
    fn from(orig: data::Transaction) -> Transaction {
        Transaction {
            date: orig.date,
            effective_date: orig.effective_date,
            clear_state: orig.clear_state,
            code: orig.code,
            payee: orig.payee,
            metadata: Vec::new(),
            posts: orig.posts.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// Posting in a transaction to represent a particular account amount increase / decrease.
pub struct Posting {
    /// Account of the post target.
    pub account: String,
    /// Posting specific ClearState.
    pub clear_state: ClearState,
    /// Amount of the posting.
    pub amount: Option<PostingAmount>,
    /// Balance after the transaction of the specified account.
    pub balance: Option<expr::ValueExpr>,
    /// Metadata information such as comment or tag.
    pub metadata: Vec<Metadata>,
}

impl Posting {
    pub fn new(account: String) -> Posting {
        Posting {
            account,
            clear_state: ClearState::default(),
            amount: None,
            balance: None,
            metadata: Vec::new(),
        }
    }
}

impl From<data::Posting> for Posting {
    fn from(orig: data::Posting) -> Posting {
        let metadata = orig
            .payee
            .into_iter()
            .map(|v| Metadata::KeyValueTag {
                key: "Payee".to_string(),
                value: v,
            })
            .collect();
        Posting {
            account: orig.account,
            clear_state: orig.clear_state,
            amount: orig.amount.map(Into::into),
            balance: orig.balance.map(Into::into),
            metadata,
        }
    }
}

/// Metadata represents meta information associated with transactions / posts.
#[derive(Debug, PartialEq, Eq)]
pub enum Metadata {
    /// Comment, which covers just one line (without the suceeding new line).
    Comment(String),
    /// Tags of word, in a format :tag1:tag2:tag3:, each tag can't contain white spaces.
    WordTags(Vec<String>),
    /// Key-value paired tag. Key can't contain white spaces.
    KeyValueTag { key: String, value: String },
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Metadata::WordTags(tags) => {
                write!(f, ":")?;
                for tag in tags {
                    write!(f, "{}:", tag)?;
                }
            }
            Metadata::KeyValueTag { key, value } => write!(f, "{}: {}", key, value)?,
            Metadata::Comment(s) => write!(f, "{}", s)?,
        };
        Ok(())
    }
}

/// This is an amout for each posting.
/// Which contains
/// - how much the asset is increased.
/// - what was the cost in the other commodity.
/// - lot information.
#[derive(Debug, PartialEq, Eq)]
pub struct PostingAmount {
    pub amount: expr::ValueExpr,
    pub cost: Option<Exchange>,
}

impl From<data::ExchangedAmount> for PostingAmount {
    fn from(v: data::ExchangedAmount) -> Self {
        PostingAmount {
            amount: v.amount.into(),
            cost: v.exchange.map(Into::into),
        }
    }
}

/// Exchange represents the amount expressed in the different commodity.
#[derive(Debug, PartialEq, Eq)]
pub enum Exchange {
    /// Specified value equals to the total amount.
    /// For example,
    /// `200 JPY @@ 2 USD`
    /// means the amount was 200 JPY, which is equal to 2 USD.
    Total(expr::ValueExpr),
    /// Specified value equals to the amount of one original commodity.
    /// For example,
    /// `200 JPY @ (1 / 100 USD)`
    /// means the amount was 200 JPY, where 1 JPY is equal to 1/100 USD.
    Rate(expr::ValueExpr),
}

impl From<data::Exchange> for Exchange {
    fn from(v: data::Exchange) -> Self {
        match v {
            data::Exchange::Total(total) => Exchange::Total(total.into()),
            data::Exchange::Rate(rate) => Exchange::Rate(rate.into()),
        }
    }
}