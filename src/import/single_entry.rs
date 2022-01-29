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

    /// ClearState, useful to overwrite default convention (if dest_account is set).
    clear_state: Option<data::ClearState>,

    /// amount in exchanged rate.
    transferred_amount: Option<data::ExchangedAmount>,

    /// Amount of the transaction, applied for the associated account.
    /// For bank account, positive means deposit, negative means withdraw.
    /// For credit card account, negative means expense, positive means payment to the card.
    amount: data::Amount,

    /// Rate of the amount, useful if the statement amount is in foreign currency.
    rate: Option<data::Exchange>,

    balance: Option<data::Amount>,

    charges: Vec<Charge>,
}

struct Charge {
    payee: String,
    amount: data::Amount,
}

impl Txn {
    pub fn new(date: NaiveDate, payee: &str, amount: data::Amount) -> Txn {
        Txn {
            date,
            effective_date: None,
            code: None,
            payee: payee.to_string(),
            dest_account: None,
            clear_state: None,
            transferred_amount: None,
            amount,
            rate: None,
            balance: None,
            charges: Vec::new(),
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

    pub fn clear_state(&mut self, clear_state: data::ClearState) -> &mut Txn {
        self.clear_state = Some(clear_state);
        self
    }

    pub fn transferred_amount(&mut self, amount: data::ExchangedAmount) -> &mut Txn {
        self.transferred_amount = Some(amount);
        self
    }

    pub fn rate(&mut self, rate: data::Exchange) -> &mut Txn {
        self.rate = Some(rate);
        self
    }

    pub fn try_add_charge_not_included<'a>(
        &'a mut self,
        payee: &str,
        amount: data::Amount,
    ) -> Result<&'a mut Txn, ImportError> {
        if amount.commodity != self.amount.commodity {
            return Err(ImportError::Unimplemented(
                "different commodity charge not supported",
            ));
        }
        if self.transferred_amount.is_some() {
            return Err(ImportError::Unimplemented(
                "already set transferred_amount isn't supported",
            ));
        }
        self.transferred_amount(data::ExchangedAmount {
            amount: data::Amount {
                value: self.amount.value - amount.value,
                commodity: amount.commodity.clone(),
            },
            exchange: None,
        });
        self.charges.push(Charge {
            payee: payee.to_string(),
            amount,
        });
        Ok(self)
    }

    pub fn add_charge<'a>(&'a mut self, payee: &str, amount: data::Amount) -> &'a mut Txn {
        self.charges.push(Charge {
            payee: payee.to_string(),
            amount,
        });
        self
    }

    fn amount(&self) -> data::ExchangedAmount {
        data::ExchangedAmount {
            amount: self.amount.clone(),
            exchange: self.rate.clone(),
        }
    }

    fn dest_amount(&self) -> data::ExchangedAmount {
        self.transferred_amount
            .as_ref()
            .map(|x| data::ExchangedAmount {
                amount: -x.amount.clone(),
                exchange: x.exchange.clone(),
            })
            .unwrap_or(data::ExchangedAmount {
                amount: -self.amount.clone(),
                exchange: None,
            })
    }

    pub fn balance(&mut self, balance: data::Amount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    pub fn to_double_entry(&self, src_account: &str) -> Result<data::Transaction, ImportError> {
        let mut posts = Vec::new();
        let post_clear = self.clear_state.unwrap_or(match &self.dest_account {
            Some(_) => data::ClearState::Uncleared,
            None => data::ClearState::Pending,
        });
        if self.amount.is_sign_positive() {
            posts.push(data::Post {
                account: self
                    .dest_account
                    .clone()
                    .unwrap_or_else(|| "Income:Unknown".to_string()),
                clear_state: post_clear,
                amount: Some(self.dest_amount()),
                balance: None,
                payee: None,
            });
            posts.push(data::Post {
                account: src_account.to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: Some(self.amount()),
                balance: self.balance.clone(),
                payee: None,
            });
        } else if self.amount.is_sign_negative() {
            posts.push(data::Post {
                account: src_account.to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: Some(self.amount()),
                balance: self.balance.clone(),
                payee: None,
            });
            posts.push(data::Post {
                account: self
                    .dest_account
                    .clone()
                    .unwrap_or_else(|| "Expenses:Unknown".to_string()),
                clear_state: post_clear,
                amount: Some(self.dest_amount()),
                balance: None,
                payee: None,
            });
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        for chrg in &self.charges {
            posts.push(data::Post {
                account: "Expenses:Commissions".to_string(),
                clear_state: data::ClearState::Uncleared,
                amount: Some(data::ExchangedAmount {
                    amount: -chrg.amount.clone(),
                    exchange: None,
                }),
                balance: None,
                payee: Some(chrg.payee.clone()),
            });
        }
        Ok(data::Transaction {
            effective_date: self.effective_date,
            clear_state: data::ClearState::Cleared,
            code: self.code.clone(),
            posts,
            ..data::Transaction::new(self.date, self.payee.clone())
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
