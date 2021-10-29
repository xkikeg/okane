use super::ImportError;
use crate::data;

use chrono::NaiveDate;

/// Represents single-entry transaction, associated with the particular account.
pub struct Txn {
    /// Date when the transaction happened.
    date: NaiveDate,

    /// Date when the transaction took effects (i.e. actually paid / transfered).
    effective_date: Option<NaiveDate>,

    /// Code of the transcation for tracking.
    code: Option<String>,

    /// Payee (or payer) of the transaction.
    payee: String,

    /// Destination account.
    dest_account: Option<String>,

    /// Amount of the transaction, applied for the associated account.
    /// For bank account, positive means deposit, negative means withdraw.
    /// For credit card account, negative means expense, positive means payment to the card.
    amount: data::Amount,

    balance: Option<data::Amount>,
}

impl Txn {
    pub fn new(date: NaiveDate, payee: &str, amount: data::Amount) -> Txn {
        Txn {
            date,
            effective_date: None,
            code: None,
            payee: payee.to_string(),
            dest_account: None,
            amount,
            balance: None,
        }
    }

    /// Set effective date, only when it's different from date.
    pub fn effective_date(&mut self, effective_date: NaiveDate) -> &mut Txn {
        if self.date != effective_date {
            self.effective_date = Some(effective_date);
        }
        self
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
            effective_date: self.effective_date,
            clear_state: data::ClearState::Cleared,
            code: self.code.clone(),
            payee: self.payee.clone(),
            posts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn test_effective_date_not_set_same_date() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd(2021, 10, 1),
            "foo",
            data::Amount {
                commodity: "JPY".to_string(),
                value: dec!(10),
            },
        );
        txn.effective_date(NaiveDate::from_ymd(2021, 10, 1));

        assert_eq!(txn.effective_date, None);
    }

    #[test]
    fn test_effective_date_set_different_date() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd(2021, 10, 1),
            "foo",
            data::Amount {
                commodity: "JPY".to_string(),
                value: dec!(10),
            },
        );
        txn.effective_date(NaiveDate::from_ymd(2021, 10, 2));

        assert_eq!(txn.effective_date, Some(NaiveDate::from_ymd(2021, 10, 2)));
    }
}
