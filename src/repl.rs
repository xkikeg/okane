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
    /// Transaction
    Txn(Transaction),
    /// Comment, not limited to one-line oppose to `Metadata`.
    Comment(TopLevelComment),
    /// Apply tag directive.
    ApplyTag(ApplyTag),
    /// "end apply tag" directive.
    EndApplyTag,
    /// "include" directive.
    Include(IncludeFile),
    /// "account" directive.
    Account(AccountDeclaration),
    /// "commodity" directive.
    Commodity(CommodityDeclaration),
}

/// Top-level comment. OK to have multi-line comment.
#[derive(Debug, PartialEq, Eq)]
pub struct TopLevelComment(String);

/// "apply tag" directive content.
#[derive(Debug, PartialEq, Eq)]
pub struct ApplyTag {
    pub key: String,
    pub value: Option<String>,
}

/// "include" directive, taking a path as an argument.
/// Path can be a relative path or an absolute path.
#[derive(Debug, PartialEq, Eq)]
pub struct IncludeFile(String);

/// "account" directive to declare account information.
#[derive(Debug, PartialEq, Eq)]
pub struct AccountDeclaration {
    /// Canonical name of the account.
    name: String,
    /// sub-directives for the account.
    details: Vec<AccountDetail>,
}

/// Sub directives for "account" directive.
#[derive(Debug, PartialEq, Eq)]
pub enum AccountDetail {
    /// Comment is a pure comment without any semantics, similar to `TopLevelComment`.
    Comment(String),
    /// Note is a "note" sub-directive.
    /// Usually it would be one-line.
    Note(String),
    /// Declare the given string is an alias for the declared account.
    Alias(String),
}

/// "commodity" directive to declare commodity information.
#[derive(Debug, PartialEq, Eq)]
pub struct CommodityDeclaration {
    /// Canonical name of the commodity.
    name: String,
    /// sub-directives for the commodity.
    details: Vec<CommodityDetail>,
}

/// Sub directives for "commodity" directive.
#[derive(Debug, PartialEq, Eq)]
pub enum CommodityDetail {
    /// Comment is a pure comment without any semantics, similar to `TopLevelComment`.
    Comment(String),
    /// Note is a "note" sub-directive to note the commodity.
    /// Usually it would be one-line.
    Note(String),
    /// Declare the given string is an alias for the declared account.
    /// Multiple declaration should work.
    Alias(String),
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
    pub lot: Lot,
}

impl From<expr::ValueExpr> for PostingAmount {
    fn from(v: expr::ValueExpr) -> Self {
        PostingAmount {
            amount: v,
            cost: None,
            lot: Lot::default(),
        }
    }
}

impl From<data::ExchangedAmount> for PostingAmount {
    fn from(v: data::ExchangedAmount) -> Self {
        PostingAmount {
            amount: v.amount.into(),
            cost: v.exchange.map(Into::into),
            lot: Lot::default(),
        }
    }
}

/// Lot information is a set of metadata to record the original lot which the commodity is acquired with.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Lot {
    pub price: Option<Exchange>,
    pub date: Option<NaiveDate>,
    pub note: Option<String>,
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
