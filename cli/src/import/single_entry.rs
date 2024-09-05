use std::{borrow::Cow, collections::HashMap};

use chrono::NaiveDate;
use rust_decimal::Decimal;

use okane_core::repl::{self, pretty_decimal::PrettyDecimal};

use super::amount::OwnedAmount;
use super::ImportError;

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
    clear_state: Option<repl::ClearState>,

    /// amount in exchanged rate.
    transferred_amount: Option<OwnedAmount>,

    /// Amount of the transaction, applied for the associated account.
    /// For bank account, positive means deposit, negative means withdraw.
    /// For credit card account, negative means expense, positive means payment to the card.
    amount: OwnedAmount,

    /// Rate of the given commodity, useful if the statement amount is in foreign currency.
    rates: HashMap<String, OwnedAmount>,

    balance: Option<OwnedAmount>,

    charges: Vec<Charge>,
}

struct Charge {
    payee: String,
    amount: OwnedAmount,
}

impl Txn {
    pub fn new(date: NaiveDate, payee: &str, amount: OwnedAmount) -> Txn {
        Txn {
            date,
            effective_date: None,
            code: None,
            payee: payee.to_string(),
            dest_account: None,
            clear_state: None,
            transferred_amount: None,
            amount,
            rates: HashMap::new(),
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

    pub fn clear_state(&mut self, clear_state: repl::ClearState) -> &mut Txn {
        self.clear_state = Some(clear_state);
        self
    }

    pub fn transferred_amount(&mut self, amount: OwnedAmount) -> &mut Txn {
        self.transferred_amount = Some(amount);
        self
    }

    pub fn add_rate(&mut self, key: CommodityPair, rate: Decimal) -> Result<&mut Txn, ImportError> {
        match self.rates.insert(
            key.target.clone(),
            OwnedAmount {
                value: rate,
                commodity: key.source.clone(),
            },
        ) {
            Some(existing) if (&existing.commodity, existing.value) != (&key.source, rate) => {
                Err(ImportError::Other(format!(
                    "given commodity {} has two distinct rates: @ {} {} and @ {} {}",
                    key.target, existing.value, existing.commodity, key.source, rate
                )))
            }
            _ => Ok(self),
        }
    }

    fn rate(&self, target: &str) -> Option<repl::Exchange> {
        self.rates
            .get(target)
            .map(|x| repl::Exchange::Rate(as_syntax_amount(x).into()))
    }

    pub fn try_add_charge_not_included<'a>(
        &'a mut self,
        payee: &str,
        amount: OwnedAmount,
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
        self.transferred_amount(OwnedAmount {
            value: self.amount.value + amount.value,
            commodity: amount.commodity.clone(),
        });
        self.charges.push(Charge {
            payee: payee.to_string(),
            amount,
        });
        Ok(self)
    }

    pub fn add_charge<'a>(&'a mut self, payee: &str, amount: OwnedAmount) -> &'a mut Txn {
        self.charges.push(Charge {
            payee: payee.to_string(),
            amount,
        });
        self
    }

    fn amount(&self) -> repl::plain::PostingAmount {
        repl::PostingAmount {
            amount: as_syntax_amount(&self.amount).into(),
            cost: self.rate(&self.amount.commodity),
            lot: repl::Lot::default(),
        }
    }

    fn dest_amount(&self) -> repl::plain::PostingAmount {
        self.transferred_amount
            .as_ref()
            .map(|transferred| repl::PostingAmount {
                // transferred_amount can be absolute value, or signed value.
                // Assuming all commodities are "positive",
                // it should have the opposite sign of the original amount.
                amount: amount_with_sign(transferred, -self.amount.value).into(),
                cost: self.rate(&transferred.commodity),
                lot: repl::Lot::default(),
            })
            .unwrap_or(to_posting_amount(negate_amount(as_syntax_amount(
                &self.amount,
            ))))
    }

    pub fn balance(&mut self, balance: OwnedAmount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    pub fn to_double_entry<'a>(
        &'a self,
        src_account: &'a str,
    ) -> Result<repl::plain::Transaction<'a>, ImportError> {
        let mut posts: Vec<repl::plain::Posting> = Vec::new();
        let post_clear = self.clear_state.unwrap_or(match &self.dest_account {
            Some(_) => repl::ClearState::Uncleared,
            None => repl::ClearState::Pending,
        });
        if self.amount.is_sign_positive() {
            posts.push(repl::Posting {
                clear_state: post_clear,
                amount: Some(self.dest_amount()),
                ..repl::Posting::new(self.dest_account.as_deref().unwrap_or("Income:Unknown"))
            });
            posts.push(repl::Posting {
                clear_state: repl::ClearState::Uncleared,
                amount: Some(self.amount()),
                balance: self.balance.as_ref().map(|x| as_syntax_amount(x).into()),
                ..repl::Posting::new(src_account)
            });
        } else if self.amount.is_sign_negative() {
            posts.push(repl::Posting {
                clear_state: repl::ClearState::Uncleared,
                amount: Some(self.amount()),
                balance: self.balance.as_ref().map(|x| as_syntax_amount(x).into()),
                ..repl::Posting::new(src_account)
            });
            posts.push(repl::Posting {
                clear_state: post_clear,
                amount: Some(self.dest_amount()),
                ..repl::Posting::new(self.dest_account.as_deref().unwrap_or("Expenses:Unknown"))
            });
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        for chrg in &self.charges {
            posts.push(repl::Posting {
                clear_state: repl::ClearState::Uncleared,
                amount: Some(to_posting_amount(as_syntax_amount(&chrg.amount))),
                balance: None,
                metadata: vec![repl::Metadata::KeyValueTag {
                    key: Cow::Borrowed("Payee"),
                    value: repl::MetadataValue::Text(chrg.payee.as_str().into()),
                }],
                ..repl::Posting::new("Expenses:Commissions")
            });
        }
        Ok(repl::Transaction {
            effective_date: self.effective_date,
            clear_state: repl::ClearState::Cleared,
            code: self.code.as_deref().map(Into::into),
            posts,
            ..repl::Transaction::new(self.date, &self.payee)
        })
    }
}

/// Pair of commodity, used for rate computation.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CommodityPair {
    pub source: String,
    pub target: String,
}

fn as_syntax_amount(amount: &OwnedAmount) -> repl::expr::Amount {
    repl::expr::Amount {
        // TODO: pass the right format.
        value: PrettyDecimal::unformatted(amount.value),
        commodity: Cow::Borrowed(&amount.commodity),
    }
}

fn negate_amount(mut amount: repl::expr::Amount) -> repl::expr::Amount {
    amount.value = -amount.value;
    amount
}

fn to_posting_amount(amount: repl::expr::Amount) -> repl::plain::PostingAmount {
    repl::PostingAmount {
        amount: amount.into(),
        cost: None,
        lot: repl::Lot::default(),
    }
}

fn amount_with_sign(amount: &OwnedAmount, sign: Decimal) -> repl::expr::Amount {
    let mut ret = as_syntax_amount(amount);
    ret.value.set_sign_positive(sign.is_sign_positive());
    ret
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
            OwnedAmount {
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
            OwnedAmount {
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

    fn syntax_amount(value: PrettyDecimal, commodity: &str) -> repl::expr::ValueExpr {
        repl::expr::Amount {
            value,
            commodity: commodity.into(),
        }
        .into()
    }

    fn owned_amount(value: Decimal, commodity: &str) -> OwnedAmount {
        OwnedAmount {
            commodity: commodity.to_string(),
            value,
        }
    }

    #[test]
    fn dest_amount_plain() {
        let txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(10), "JPY"),
        );

        assert_eq!(
            txn.dest_amount(),
            syntax_amount(PrettyDecimal::unformatted(dec!(-10)), "JPY").into(),
        );
    }

    #[test]
    fn dest_amount_exchanged() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );
        txn.add_rate(
            CommodityPair {
                source: "JPY".to_owned(),
                target: "USD".to_owned(),
            },
            dec!(100),
        )
        .unwrap();
        txn.transferred_amount(owned_amount(dec!(10.00), "USD"));

        assert_eq!(
            txn.dest_amount(),
            repl::PostingAmount {
                amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                cost: Some(repl::Exchange::Rate(syntax_amount(
                    PrettyDecimal::unformatted(dec!(100)),
                    "JPY"
                ))),
                lot: repl::Lot::default(),
            },
        )
    }

    #[test]
    fn dest_amount_transferred_negative() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );
        txn.add_rate(
            CommodityPair {
                source: "JPY".to_owned(),
                target: "USD".to_owned(),
            },
            dec!(100),
        )
        .unwrap();
        txn.transferred_amount(owned_amount(dec!(-10.00), "USD"));

        assert_eq!(
            txn.dest_amount(),
            repl::PostingAmount {
                amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                cost: Some(repl::Exchange::Rate(syntax_amount(
                    PrettyDecimal::unformatted(dec!(100)),
                    "JPY"
                ))),
                lot: repl::Lot::default(),
            },
        )
    }
}
