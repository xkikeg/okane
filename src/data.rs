//! data contains fundamental types used in Ledger data.
//! Note the structure is quite similar to repl module,
//! however, repl is for textual representation while
//! data is more for understanding.

use std::cmp::Ordering;

use rust_decimal::Decimal;

/// Represents a transaction where the money transfered across the accounts.
#[derive(Debug, PartialEq)]
pub struct Transaction {
    /// Date when the transaction issued.
    pub date: chrono::NaiveDate,
    /// Date when the transaction got effective, optional.
    pub effective_date: Option<chrono::NaiveDate>,
    /// Indiacates clearing state of the entire transaction.
    pub clear_state: ClearState,
    /// Transaction code (not necessarily unique).
    pub code: Option<String>,
    /// Label of the transaction, often the opposite party of the transaction.
    pub payee: String,
    /// Postings of the transaction, could be empty.
    pub posts: Vec<Posting>,
}

impl Transaction {
    /// Constructs minimal transaction.
    pub fn new(date: chrono::NaiveDate, payee: String) -> Transaction {
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

/// Represents a clearing state, often combined with the ambiguity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClearState {
    /// No specific meaning.
    Uncleared,
    /// Useful to declare that the transaction / post is confirmed.
    Cleared,
    /// Useful to declare that the transaction / post is still pending.
    Pending,
}

impl Default for ClearState {
    fn default() -> ClearState {
        ClearState::Uncleared
    }
}

/// Posting in a transaction to represent a particular account amount increase / decrease.
#[derive(Debug, PartialEq)]
pub struct Posting {
    /// Account of the post target.
    pub account: String,
    /// Posting specific ClearState.
    pub clear_state: ClearState,
    /// Amount of the posting.
    pub amount: Option<ExchangedAmount>,
    /// Balance after the transaction of the specified account.
    pub balance: Option<Amount>,
    /// Overwrites the payee.
    pub payee: Option<String>,
}

/// Cost of the posting.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExchangedAmount {
    /// Amount that posting account was increased by.
    pub amount: Amount,
    /// Exchange rate information to balance with other postings.
    pub exchange: Option<Exchange>,
}

/// Commodity exchange information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Exchange {
    /// Represents the amount equals to the `ExchangedAmount.amount`.
    Total(Amount),
    /// Represents te amount equals to 1 `ExchangedAmount.amount.commodity`.
    Rate(Amount),
}

/// Amount of posting, balance, ...
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount {
    /// Numerical value.
    pub value: Decimal,
    /// Commodity aka currency.
    pub commodity: String,
}

impl Amount {
    /// Returns `true` if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    /// Returns `true` if the amount is positive.
    pub fn is_sign_positive(&self) -> bool {
        self.value.is_sign_positive()
    }

    /// Returns `true` if the amount is negative.
    pub fn is_sign_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
}

/// # Examples
///
/// ```
/// # use rust_decimal_macros::dec;
/// let x = okane::data::Amount{
///     value: dec!(-5),
///     commodity: "JPY".to_string(),
/// };
/// let y = -x.clone();
/// assert_eq!(x.value, dec!(-5));
/// assert_eq!(x.commodity, "JPY");
/// assert_eq!(y.value, dec!(5));
/// assert_eq!(y.commodity, "JPY");
/// ```
impl std::ops::Neg for Amount {
    type Output = Amount;
    fn neg(self) -> Amount {
        Amount {
            value: -self.value,
            commodity: self.commodity,
        }
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.commodity != other.commodity {
            None
        } else {
            Some(self.value.cmp(&other.value))
        }
    }
}
