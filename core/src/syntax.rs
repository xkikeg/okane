//! Provides Ledger file format syntax representation.

pub mod decoration;
pub mod display;
pub mod expr;
pub mod plain;
pub mod pretty_decimal;
pub mod tracked;

use std::{borrow::Cow, fmt};

use bounded_static::ToStatic;
use chrono::{NaiveDate, NaiveDateTime};
use derive_where::derive_where;

#[cfg(test)]
use bounded_static::ToBoundedStatic;

use decoration::Decoration;

/// Top-level entry of the LedgerFile.
#[derive_where(Debug, PartialEq, Eq)]
pub enum LedgerEntry<'i, Deco: Decoration> {
    /// Transaction
    Txn(Transaction<'i, Deco>),
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

impl<'i> LedgerEntry<'i, plain::Ident> {
    #[cfg(test)]
    pub(crate) fn to_static(&self) -> LedgerEntry<'static, plain::Ident> {
        match self {
            LedgerEntry::Txn(v) => LedgerEntry::Txn(v.to_static()),
            LedgerEntry::Comment(v) => LedgerEntry::Comment(v.to_static()),
            LedgerEntry::ApplyTag(v) => LedgerEntry::ApplyTag(v.to_static()),
            LedgerEntry::EndApplyTag => LedgerEntry::EndApplyTag,
            LedgerEntry::Include(v) => LedgerEntry::Include(v.to_static()),
            LedgerEntry::Account(v) => LedgerEntry::Account(v.to_static()),
            LedgerEntry::Commodity(v) => LedgerEntry::Commodity(v.to_static()),
        }
    }
}

/// Top-level comment. OK to have multi-line comment.
#[derive(Debug, PartialEq, Eq, ToStatic)]
pub struct TopLevelComment<'i>(pub Cow<'i, str>);

/// "apply tag" directive content.
#[derive(Debug, PartialEq, Eq, ToStatic)]
pub struct ApplyTag<'i> {
    pub key: Cow<'i, str>,
    pub value: Option<MetadataValue<'i>>,
}

/// "include" directive, taking a path as an argument.
/// Path can be a relative path or an absolute path.
#[derive(Debug, PartialEq, Eq, ToStatic)]
pub struct IncludeFile<'i>(pub Cow<'i, str>);

/// "account" directive to declare account information.
#[derive(Debug, PartialEq, Eq, ToStatic)]
pub struct AccountDeclaration<'i> {
    /// Canonical name of the account.
    pub name: Cow<'i, str>,
    /// sub-directives for the account.
    pub details: Vec<AccountDetail<'i>>,
}

/// Sub directives for "account" directive.
#[derive(Debug, PartialEq, Eq, ToStatic)]
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
#[derive(Debug, PartialEq, Eq, ToStatic)]
pub struct CommodityDeclaration<'i> {
    /// Canonical name of the commodity.
    pub name: Cow<'i, str>,
    /// sub-directives for the commodity.
    pub details: Vec<CommodityDetail<'i>>,
}

/// Sub directives for "commodity" directive.
#[derive(Debug, PartialEq, Eq, ToStatic)]
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
#[derive_where(Debug, PartialEq, Eq)]
pub struct Transaction<'i, Deco: Decoration> {
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
    pub posts: Vec<Deco::Decorated<Posting<'i, Deco>>>,
    /// Transaction level metadata.
    pub metadata: Vec<Metadata<'i>>,
}

impl<'i> Transaction<'i, plain::Ident> {
    #[cfg(test)]
    fn to_static(&self) -> Transaction<'static, plain::Ident> {
        let mut posts = Vec::new();
        for p in &self.posts {
            posts.push(p.to_static());
        }
        Transaction {
            date: self.date,
            effective_date: self.effective_date,
            clear_state: self.clear_state,
            code: self.code.to_static(),
            payee: self.payee.to_static(),
            posts,
            metadata: self.metadata.to_static(),
        }
    }
}

impl<'i, Deco: Decoration> Transaction<'i, Deco> {
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

#[derive_where(Debug, PartialEq, Eq)]
/// Posting in a transaction to represent a particular account amount increase / decrease.
pub struct Posting<'i, Deco: Decoration> {
    /// Account of the post target.
    pub account: Cow<'i, str>,
    /// Posting specific ClearState.
    pub clear_state: ClearState,
    /// Amount of the posting.
    pub amount: Option<PostingAmount<'i, Deco>>,
    /// Balance after the transaction of the specified account.
    pub balance: Option<Deco::Decorated<expr::ValueExpr<'i>>>,
    /// Metadata information such as comment or tag.
    pub metadata: Vec<Metadata<'i>>,
}

impl<'i> Posting<'i, plain::Ident> {
    #[cfg(test)]
    fn to_static(&self) -> Posting<'static, plain::Ident> {
        Posting {
            account: self.account.to_static(),
            clear_state: self.clear_state,
            amount: self.amount.as_ref().map(|x| x.to_static()),
            balance: self.balance.to_static(),
            metadata: self.metadata.to_static(),
        }
    }
}

impl<'i, Deco: Decoration> Posting<'i, Deco> {
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

/// Represents a clearing state, often combined with the ambiguity.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, ToStatic)]
pub enum ClearState {
    /// No specific meaning.
    #[default]
    Uncleared,
    /// Useful to declare that the transaction / post is confirmed.
    Cleared,
    /// Useful to declare that the transaction / post is still pending.
    Pending,
}

/// Metadata represents meta information associated with transactions / posts.
#[derive(Debug, PartialEq, Eq, ToStatic)]
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
#[derive(Debug, PartialEq, Eq, ToStatic)]
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
#[derive_where(Debug, PartialEq, Eq)]
pub struct PostingAmount<'i, Deco: Decoration> {
    pub amount: Deco::Decorated<expr::ValueExpr<'i>>,
    pub cost: Option<Deco::Decorated<Exchange<'i>>>,
    pub lot: Lot<'i, Deco>,
}

impl<'i> PostingAmount<'i, plain::Ident> {
    #[cfg(test)]
    fn to_static(&self) -> PostingAmount<'static, plain::Ident> {
        PostingAmount {
            amount: self.amount.to_static(),
            cost: self.cost.to_static(),
            lot: self.lot.to_static(),
        }
    }
}

impl<'i> From<expr::ValueExpr<'i>> for PostingAmount<'i, plain::Ident> {
    fn from(v: expr::ValueExpr<'i>) -> Self {
        PostingAmount {
            amount: v,
            cost: None,
            lot: Lot::default(),
        }
    }
}

/// Lot information is a set of metadata to record the original lot which the commodity is acquired with.
#[derive_where(Debug, PartialEq, Eq)]
pub struct Lot<'i, Deco: Decoration> {
    pub price: Option<Deco::Decorated<Exchange<'i>>>,
    pub date: Option<NaiveDate>,
    pub note: Option<Cow<'i, str>>,
}

impl<'i> Lot<'i, plain::Ident> {
    #[cfg(test)]
    fn to_static(&self) -> Lot<'static, plain::Ident> {
        Lot {
            price: self.price.to_static(),
            date: self.date.to_static(),
            note: self.note.to_static(),
        }
    }
}

impl<'i, Deco: Decoration> Default for Lot<'i, Deco> {
    fn default() -> Self {
        Self {
            price: None,
            date: None,
            note: None,
        }
    }
}

/// Exchange represents the amount expressed in the different commodity.
#[derive(Debug, PartialEq, Eq, ToStatic)]
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

/// Price DB entry, which contains one commodity price
/// in another commodity on a particular date time.
#[derive(Debug, PartialEq, Eq, ToStatic)]
pub struct PriceDBEntry<'i> {
    pub datetime: NaiveDateTime,
    /// Target commodity of the price.
    pub target: Cow<'i, str>,
    /// The rate of the target commodity.
    pub rate: expr::ValueExpr<'i>,
}
