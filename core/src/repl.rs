//! repl represents Ledger file format representation.
//! Note the structure is quite similar to data module,
//! however, repl is for textual representation while
//! data is more for understanding.

pub mod display;
pub mod expr;
pub mod pretty_decimal;

use crate::datamodel;
pub use crate::datamodel::ClearState;

use std::{borrow::Cow, fmt};

use chrono::NaiveDate;

/// Top-level entry of the LedgerFile.
#[derive(Debug, PartialEq, Eq)]
pub enum LedgerEntry<'i> {
    /// Transaction
    Txn(Transaction<'i>),
    /// Comment, not limited to one-line oppose to `Metadata`.
    Comment(TopLevelComment<'i>),
    /// Apply tag directive.
    ApplyTag(ApplyTag<'i>),
    /// "end apply tag" directive.
    EndApplyTag,
    /// "include" directive.
    Include(IncludeFile<'i>),
    /// "account" directive.
    Account(AccountDeclaration<'i>),
    /// "commodity" directive.
    Commodity(CommodityDeclaration<'i>),
}

/// Top-level comment. OK to have multi-line comment.
#[derive(Debug, PartialEq, Eq)]
pub struct TopLevelComment<'i>(pub Cow<'i, str>);

/// "apply tag" directive content.
#[derive(Debug, PartialEq, Eq)]
pub struct ApplyTag<'i> {
    pub key: Cow<'i, str>,
    pub value: Option<MetadataValue<'i>>,
}

/// "include" directive, taking a path as an argument.
/// Path can be a relative path or an absolute path.
#[derive(Debug, PartialEq, Eq)]
pub struct IncludeFile<'i>(pub Cow<'i, str>);

/// "account" directive to declare account information.
#[derive(Debug, PartialEq, Eq)]
pub struct AccountDeclaration<'i> {
    /// Canonical name of the account.
    pub name: Cow<'i, str>,
    /// sub-directives for the account.
    pub details: Vec<AccountDetail<'i>>,
}

/// Sub directives for "account" directive.
#[derive(Debug, PartialEq, Eq)]
pub enum AccountDetail<'i> {
    /// Comment is a pure comment without any semantics, similar to `TopLevelComment`.
    Comment(Cow<'i, str>),
    /// Note is a "note" sub-directive.
    /// Usually it would be one-line.
    Note(Cow<'i, str>),
    /// Declare the given string is an alias for the declared account.
    Alias(Cow<'i, str>),
}

/// "commodity" directive to declare commodity information.
#[derive(Debug, PartialEq, Eq)]
pub struct CommodityDeclaration<'i> {
    /// Canonical name of the commodity.
    pub name: Cow<'i, str>,
    /// sub-directives for the commodity.
    pub details: Vec<CommodityDetail<'i>>,
}

/// Sub directives for "commodity" directive.
#[derive(Debug, PartialEq, Eq)]
pub enum CommodityDetail<'i> {
    /// Comment is a pure comment without any semantics, similar to `TopLevelComment`.
    Comment(Cow<'i, str>),
    /// Note is a "note" sub-directive to note the commodity.
    /// Usually it would be one-line.
    Note(Cow<'i, str>),
    /// Declare the given string is an alias for the declared account.
    /// Multiple declaration should work.
    Alias(Cow<'i, str>),
    /// Format describes how the comodity should be printed.
    Format(expr::Amount<'i>),
}

/// Represents a transaction where the money transfered across the accounts.
#[derive(Debug, PartialEq, Eq)]
pub struct Transaction<'i> {
    /// Date when the transaction issued.
    pub date: NaiveDate,
    /// Date when the transaction got effective, optional.
    pub effective_date: Option<NaiveDate>,
    /// Indiacates clearing state of the entire transaction.
    pub clear_state: ClearState,
    /// Transaction code (not necessarily unique).
    pub code: Option<Cow<'i, str>>,
    /// Label of the transaction, often the opposite party of the transaction.
    pub payee: Cow<'i, str>,
    /// Postings of the transaction, could be empty.
    pub posts: Vec<Posting<'i>>,
    /// Transaction level metadata.
    pub metadata: Vec<Metadata<'i>>,
}

impl<'i> Transaction<'i> {
    /// Constructs minimal transaction.
    pub fn new<T>(date: NaiveDate, payee: T) -> Self
    where
        T: Into<Cow<'i, str>>,
    {
        Transaction {
            date,
            effective_date: None,
            clear_state: ClearState::Uncleared,
            code: None,
            payee: payee.into(),
            metadata: Vec::new(),
            posts: Vec::new(),
        }
    }
}

impl<'i> From<&'i datamodel::Transaction> for Transaction<'i> {
    fn from(orig: &'i datamodel::Transaction) -> Self {
        Transaction {
            date: orig.date,
            effective_date: orig.effective_date,
            clear_state: orig.clear_state,
            code: orig.code.as_ref().map(|x| Cow::Borrowed(x.as_str())),
            payee: (&orig.payee).into(),
            metadata: Vec::new(),
            posts: orig.posts.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// Posting in a transaction to represent a particular account amount increase / decrease.
pub struct Posting<'i> {
    /// Account of the post target.
    pub account: Cow<'i, str>,
    /// Posting specific ClearState.
    pub clear_state: ClearState,
    /// Amount of the posting.
    pub amount: Option<PostingAmount<'i>>,
    /// Balance after the transaction of the specified account.
    pub balance: Option<expr::ValueExpr<'i>>,
    /// Metadata information such as comment or tag.
    pub metadata: Vec<Metadata<'i>>,
}

impl<'i> Posting<'i> {
    pub fn new<T: Into<Cow<'i, str>>>(account: T) -> Self {
        Posting {
            account: account.into(),
            clear_state: ClearState::default(),
            amount: None,
            balance: None,
            metadata: Vec::new(),
        }
    }
}

impl<'i> From<&'i datamodel::Posting> for Posting<'i> {
    fn from(orig: &'i datamodel::Posting) -> Self {
        let metadata = orig
            .payee
            .iter()
            .map(|v| Metadata::KeyValueTag {
                key: Cow::Borrowed("Payee"),
                value: MetadataValue::Text(Cow::Borrowed(v.as_str())),
            })
            .collect();
        Posting {
            account: Cow::Borrowed(&orig.account),
            clear_state: orig.clear_state,
            amount: orig.amount.as_ref().map(Into::into),
            balance: orig.balance.as_ref().map(Into::into),
            metadata,
        }
    }
}

/// Metadata represents meta information associated with transactions / posts.
#[derive(Debug, PartialEq, Eq)]
pub enum Metadata<'i> {
    /// Comment, which covers just one line (without the suceeding new line).
    Comment(Cow<'i, str>),
    /// Tags of word, in a format :tag1:tag2:tag3:, each tag can't contain white spaces.
    WordTags(Vec<Cow<'i, str>>),
    /// Key-value paired tag. Key can't contain white spaces.
    KeyValueTag {
        key: Cow<'i, str>,
        value: MetadataValue<'i>,
    },
}

/// MetadataValue represents the value in key-value pair used in `Metadata`.
#[derive(Debug, PartialEq, Eq)]
pub enum MetadataValue<'i> {
    /// Regular string.
    Text(Cow<'i, str>),
    /// Expression parsed properly prefixed by `::` instead of `:`.
    // TODO: Change htis type to Expr not Cow<'i, str>.
    Expr(Cow<'i, str>),
}

/// This is an amout for each posting.
/// Which contains
/// - how much the asset is increased.
/// - what was the cost in the other commodity.
/// - lot information.
#[derive(Debug, PartialEq, Eq)]
pub struct PostingAmount<'i> {
    pub amount: expr::ValueExpr<'i>,
    pub cost: Option<Exchange<'i>>,
    pub lot: Lot<'i>,
}

impl<'i> From<expr::ValueExpr<'i>> for PostingAmount<'i> {
    fn from(v: expr::ValueExpr<'i>) -> Self {
        PostingAmount {
            amount: v,
            cost: None,
            lot: Lot::default(),
        }
    }
}

impl<'i> From<&'i datamodel::ExchangedAmount> for PostingAmount<'i> {
    fn from(v: &'i datamodel::ExchangedAmount) -> Self {
        PostingAmount {
            amount: (&v.amount).into(),
            cost: v.exchange.as_ref().map(Into::into),
            lot: Lot::default(),
        }
    }
}

/// Lot information is a set of metadata to record the original lot which the commodity is acquired with.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Lot<'i> {
    pub price: Option<Exchange<'i>>,
    pub date: Option<NaiveDate>,
    pub note: Option<Cow<'i, str>>,
}

/// Exchange represents the amount expressed in the different commodity.
#[derive(Debug, PartialEq, Eq)]
pub enum Exchange<'i> {
    /// Specified value equals to the total amount.
    /// For example,
    /// `200 JPY @@ 2 USD`
    /// means the amount was 200 JPY, which is equal to 2 USD.
    Total(expr::ValueExpr<'i>),
    /// Specified value equals to the amount of one original commodity.
    /// For example,
    /// `200 JPY @ (1 / 100 USD)`
    /// means the amount was 200 JPY, where 1 JPY is equal to 1/100 USD.
    Rate(expr::ValueExpr<'i>),
}

impl<'i> From<&'i datamodel::Exchange> for Exchange<'i> {
    fn from(v: &'i datamodel::Exchange) -> Self {
        match v {
            datamodel::Exchange::Total(total) => Exchange::Total(total.into()),
            datamodel::Exchange::Rate(rate) => Exchange::Rate(rate.into()),
        }
    }
}
