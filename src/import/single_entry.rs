use super::ImportError;
use crate::data;

use chrono::NaiveDate;

/// Represents single-entry transaction, associated with the particular account.
pub struct Txn {
    /// Date of the transaction happened.
    pub date: NaiveDate,

    /// Code of the transcation for tracking.
    pub code: Option<String>,

    /// Payee (or payer) of the transaction.
    pub payee: String,
    
    /// Destination account.
    pub dest_account: Option<String>,

    /// Amount of the transaction, applied for the associated account.
    /// For bank account, positive means deposit, negative means withdraw.
    /// For credit card account, negative means expense, positive means payment to the card.
    pub amount: data::Amount,

    pub balance: Option<data::Amount>,
}

impl Txn {
    pub fn to_double_entry(self, src_account: &str) -> Result<data::Transaction, ImportError> {
        let mut posts = Vec::new();
        let post_clear = match &self.dest_account {
            Some(_) => data::ClearState::Uncleared,
            None => data::ClearState::Pending,
        };
        if self.amount.is_sign_positive() {
            posts.push(data::Post {
                account: self.dest_account.unwrap_or("Incomes:Unknown".to_string()),
                clear_state: post_clear,
                amount: -self.amount.clone(),
                balance: None,
            });
            posts.push(data::Post {
                account: src_account.to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: self.amount,
                balance: self.balance,
            });
        } else if self.amount.is_sign_negative() {
            posts.push(data::Post {
                account: src_account.to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: self.amount.clone(),
                balance: self.balance,
            });
            posts.push(data::Post {
                account: self.dest_account.unwrap_or("Expenses:Unknown".to_string()),
                clear_state: post_clear,
                amount: -self.amount,
                balance: None,
            });
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        return Ok(data::Transaction {
            date: self.date,
            clear_state: data::ClearState::Cleared,
            code: self.code,
            payee: self.payee,
            posts: posts,
        });
    }
}
