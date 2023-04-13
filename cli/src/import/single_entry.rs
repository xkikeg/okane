use super::ImportError;
use okane_core::datamodel;

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
    clear_state: Option<datamodel::ClearState>,

    /// amount in exchanged rate.
    transferred_amount: Option<datamodel::ExchangedAmount>,

    /// Amount of the transaction, applied for the associated account.
    /// For bank account, positive means deposit, negative means withdraw.
    /// For credit card account, negative means expense, positive means payment to the card.
    amount: datamodel::Amount,

    /// Rate of the amount, useful if the statement amount is in foreign currency.
    rate: Option<datamodel::Exchange>,

    balance: Option<datamodel::Amount>,

    charges: Vec<Charge>,
}

struct Charge {
    payee: String,
    amount: datamodel::Amount,
}

impl Txn {
    pub fn new(date: NaiveDate, payee: &str, amount: datamodel::Amount) -> Txn {
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

    pub fn clear_state(&mut self, clear_state: datamodel::ClearState) -> &mut Txn {
        self.clear_state = Some(clear_state);
        self
    }

    pub fn transferred_amount(&mut self, amount: datamodel::ExchangedAmount) -> &mut Txn {
        self.transferred_amount = Some(amount);
        self
    }

    pub fn rate(&mut self, rate: datamodel::Exchange) -> &mut Txn {
        self.rate = Some(rate);
        self
    }

    pub fn try_add_charge_not_included<'a>(
        &'a mut self,
        payee: &str,
        amount: datamodel::Amount,
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
        self.transferred_amount(datamodel::ExchangedAmount {
            amount: datamodel::Amount {
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

    pub fn add_charge<'a>(&'a mut self, payee: &str, amount: datamodel::Amount) -> &'a mut Txn {
        self.charges.push(Charge {
            payee: payee.to_string(),
            amount,
        });
        self
    }

    fn amount(&self) -> datamodel::ExchangedAmount {
        datamodel::ExchangedAmount {
            amount: self.amount.clone(),
            exchange: self.rate.clone(),
        }
    }

    fn dest_amount(&self) -> datamodel::ExchangedAmount {
        self.transferred_amount
            .as_ref()
            .map(|x| datamodel::ExchangedAmount {
                amount: -x.amount.clone(),
                exchange: x.exchange.clone(),
            })
            .unwrap_or(datamodel::ExchangedAmount {
                amount: -self.amount.clone(),
                exchange: None,
            })
    }

    pub fn balance(&mut self, balance: datamodel::Amount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    pub fn to_double_entry(
        &self,
        src_account: &str,
    ) -> Result<datamodel::Transaction, ImportError> {
        let mut posts = Vec::new();
        let post_clear = self.clear_state.unwrap_or(match &self.dest_account {
            Some(_) => datamodel::ClearState::Uncleared,
            None => datamodel::ClearState::Pending,
        });
        if self.amount.is_sign_positive() {
            posts.push(datamodel::Posting {
                account: self
                    .dest_account
                    .clone()
                    .unwrap_or_else(|| "Income:Unknown".to_string()),
                clear_state: post_clear,
                amount: Some(self.dest_amount()),
                balance: None,
                payee: None,
            });
            posts.push(datamodel::Posting {
                account: src_account.to_string(),
                clear_state: datamodel::ClearState::Uncleared,
                amount: Some(self.amount()),
                balance: self.balance.clone(),
                payee: None,
            });
        } else if self.amount.is_sign_negative() {
            posts.push(datamodel::Posting {
                account: src_account.to_string(),
                clear_state: datamodel::ClearState::Uncleared,
                amount: Some(self.amount()),
                balance: self.balance.clone(),
                payee: None,
            });
            posts.push(datamodel::Posting {
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
            posts.push(datamodel::Posting {
                account: "Expenses:Commissions".to_string(),
                clear_state: datamodel::ClearState::Uncleared,
                amount: Some(datamodel::ExchangedAmount {
                    amount: -chrg.amount.clone(),
                    exchange: None,
                }),
                balance: None,
                payee: Some(chrg.payee.clone()),
            });
        }
        Ok(datamodel::Transaction {
            effective_date: self.effective_date,
            clear_state: datamodel::ClearState::Cleared,
            code: self.code.clone(),
            posts,
            ..datamodel::Transaction::new(self.date, self.payee.clone())
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
            NaiveDate::from_ymd_opt(2021, 10, 1).unwrap(),
            "foo",
            datamodel::Amount {
                commodity: "JPY".to_string(),
                value: dec!(10),
            },
        );
        txn.effective_date(NaiveDate::from_ymd_opt(2021, 10, 1).unwrap());

        assert_eq!(txn.effective_date, None);
    }

    #[test]
    fn test_effective_date_set_different_date() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2021, 10, 1).unwrap(),
            "foo",
            datamodel::Amount {
                commodity: "JPY".to_string(),
                value: dec!(10),
            },
        );
        txn.effective_date(NaiveDate::from_ymd_opt(2021, 10, 2).unwrap());

        assert_eq!(
            txn.effective_date,
            Some(NaiveDate::from_ymd_opt(2021, 10, 2).unwrap())
        );
    }
}
