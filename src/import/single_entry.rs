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
    pub fn new(date: NaiveDate, payee: &str, amount: data::Amount) -> Txn {
        Txn {
            date,
            code: None,
            payee: payee.to_string(),
            dest_account: None,
            amount,
            balance: None,
        }
    }

    pub fn code_option<'a>(&'a mut self, code: Option<&str>) -> &'a mut Txn {
        self.code = code.map(str::to_string);
        self
    }

    pub fn code<'a>(&'a mut self, code: &str) -> &'a mut Txn {
        self.code = Some(code.to_string());
        self
    }

    pub fn dest_account_option<'a>(&'a mut self, dest_account: Option<&str>) -> &'a mut Txn {
        self.dest_account = dest_account.map(str::to_string);
        self
    }

    pub fn dest_account<'a>(&'a mut self, dest_account: &str) -> &'a mut Txn {
        self.dest_account = Some(dest_account.to_string());
        self
    }

    pub fn balance(&mut self, balance: data::Amount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    pub fn to_double_entry(&self, src_account: &str) -> Result<data::Transaction, ImportError> {
        let mut posts = Vec::new();
        let post_clear = match &self.dest_account {
            Some(_) => data::ClearState::Uncleared,
            None => data::ClearState::Pending,
        };
        if self.amount.is_sign_positive() {
            posts.push(data::Post {
                account: self
                    .dest_account
                    .clone()
                    .unwrap_or_else(|| "Incomes:Unknown".to_string()),
                clear_state: post_clear,
                amount: -self.amount.clone(),
                balance: None,
            });
            posts.push(data::Post {
                account: src_account.to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: self.amount.clone(),
                balance: self.balance.clone(),
            });
        } else if self.amount.is_sign_negative() {
            posts.push(data::Post {
                account: src_account.to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: self.amount.clone(),
                balance: self.balance.clone(),
            });
            posts.push(data::Post {
                account: self
                    .dest_account
                    .clone()
                    .unwrap_or_else(|| "Expenses:Unknown".to_string()),
                clear_state: post_clear,
                amount: -self.amount.clone(),
                balance: None,
            });
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        Ok(data::Transaction {
            date: self.date,
            clear_state: data::ClearState::Cleared,
            code: self.code.clone(),
            payee: self.payee.clone(),
            posts,
        })
    }
}
