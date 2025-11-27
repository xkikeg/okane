use std::{borrow::Cow, collections::HashMap};

use chrono::NaiveDate;
use pretty_decimal::PrettyDecimal;
use rust_decimal::Decimal;

use okane_core::syntax::{self};

use super::amount::{AmountRef, BorrowedAmount, OwnedAmount};
use super::ImportError;

/// Represents single-entry transaction, associated with the particular account.
#[derive(Debug, Clone)]
pub struct Txn {
    /// Date when the transaction happened.
    date: NaiveDate,

    /// Date when the transaction took effects (i.e. actually paid / transfered).
    effective_date: Option<NaiveDate>,

    /// Code of the transcation for tracking.
    code: Option<String>,

    /// Payee (or payer) of the transaction.
    payee: String,

    comments: Vec<String>,

    /// Destination account.
    dest_account: Option<String>,

    /// ClearState, useful to overwrite default convention (if dest_account is set).
    clear_state: Option<syntax::ClearState>,

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

/// Optional object to control the output (such as commodity renaming).
#[derive(Debug, Default)]
pub struct Options {
    /// Renames the commodity in the key into the corresponding value.
    pub commodity_rename: HashMap<String, String>,
}

/// Represents the charge (commission) in the transaction.
#[derive(Debug, Clone)]
struct Charge {
    payee: String,
    amount: OwnedAmount,
}

// TODO: Allow injecting these values from config.
// https://github.com/xkikeg/okane/issues/287
const LABEL_COMMISSIONS: &str = "Expenses:Commissions";
const LABEL_UNKNOWN_INCOME: &str = "Income:Unknown";
const LABEL_UNKNOWN_EXPENSE: &str = "Expenses:Unknown";

impl Txn {
    pub fn new(date: NaiveDate, payee: &str, amount: OwnedAmount) -> Txn {
        Txn {
            date,
            effective_date: None,
            code: None,
            payee: payee.to_string(),
            comments: Vec::new(),
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

    pub fn code_option(&mut self, code: Option<String>) -> &mut Txn {
        self.code = code;
        self
    }

    pub fn add_comment(&mut self, comment: String) -> &mut Txn {
        self.comments.push(comment);
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

    pub fn clear_state(&mut self, clear_state: syntax::ClearState) -> &mut Txn {
        self.clear_state = Some(clear_state);
        self
    }

    pub fn transferred_amount(&mut self, amount: OwnedAmount) -> &mut Txn {
        self.transferred_amount = Some(amount);
        self
    }

    pub fn add_rate(&mut self, key: CommodityPair, rate: Decimal) -> Result<&mut Txn, ImportError> {
        if key.source == key.target {
            return Err(ImportError::Other(format!(
                "cannot handle rate with the same commodity {}",
                key.source
            )));
        }
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

    fn rate<'a>(&'a self, target: &str, opts: &'a Options) -> Option<syntax::Exchange<'a>> {
        self.rates
            .get(target)
            .map(|x| syntax::Exchange::Rate(as_syntax_amount(x, opts).into()))
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

    fn to_posting_amount<'a>(
        &'a self,
        amount: BorrowedAmount<'a>,
        opts: &'a Options,
    ) -> syntax::plain::PostingAmount<'a> {
        syntax::PostingAmount {
            amount: as_syntax_amount(amount, opts).into(),
            cost: self.rate(amount.commodity, opts),
            lot: syntax::Lot::default(),
        }
    }

    fn amount<'a>(&'a self, opts: &'a Options) -> syntax::plain::PostingAmount<'a> {
        self.to_posting_amount(self.amount.into_borrowed(), opts)
    }

    fn dest_amount<'a>(&'a self, opts: &'a Options) -> syntax::plain::PostingAmount<'a> {
        self.transferred_amount
            .as_ref()
            .map(|transferred| {
                self.to_posting_amount(amount_with_sign(transferred, -self.amount.value), opts)
            })
            .unwrap_or_else(|| self.to_posting_amount(-self.amount.into_borrowed(), opts))
    }

    pub fn balance(&mut self, balance: OwnedAmount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    pub fn to_double_entry<'a>(
        &'a self,
        src_account: &'a str,
        opts: &'a Options,
    ) -> Result<syntax::plain::Transaction<'a>, ImportError> {
        let mut posts: Vec<syntax::plain::Posting> = Vec::new();
        let post_clear = self.clear_state.unwrap_or(match &self.dest_account {
            Some(_) => syntax::ClearState::Uncleared,
            None => syntax::ClearState::Pending,
        });
        let add_charges = |posts: &mut Vec<syntax::plain::Posting<'a>>| {
            for chrg in &self.charges {
                posts.push(syntax::Posting {
                    clear_state: syntax::ClearState::Uncleared,
                    amount: Some(self.to_posting_amount(chrg.amount.into_borrowed(), opts)),
                    balance: None,
                    metadata: vec![syntax::Metadata::KeyValueTag {
                        key: Cow::Borrowed("Payee"),
                        value: syntax::MetadataValue::Text(chrg.payee.as_str().into()),
                    }],
                    ..syntax::Posting::new_untracked(LABEL_COMMISSIONS)
                });
            }
        };
        if self.amount.value.is_sign_positive() {
            posts.push(syntax::Posting {
                clear_state: syntax::ClearState::Uncleared,
                amount: Some(self.amount(opts)),
                balance: self
                    .balance
                    .as_ref()
                    .map(|x| as_syntax_amount(x, opts).into()),
                ..syntax::Posting::new_untracked(src_account)
            });
            add_charges(&mut posts);
            posts.push(syntax::Posting {
                clear_state: post_clear,
                amount: Some(self.dest_amount(opts)),
                ..syntax::Posting::new_untracked(
                    self.dest_account.as_deref().unwrap_or(LABEL_UNKNOWN_INCOME),
                )
            });
        } else if self.amount.value.is_sign_negative() {
            posts.push(syntax::Posting {
                clear_state: post_clear,
                amount: Some(self.dest_amount(opts)),
                ..syntax::Posting::new_untracked(
                    self.dest_account
                        .as_deref()
                        .unwrap_or(LABEL_UNKNOWN_EXPENSE),
                )
            });
            add_charges(&mut posts);
            posts.push(syntax::Posting {
                clear_state: syntax::ClearState::Uncleared,
                amount: Some(self.amount(opts)),
                balance: self
                    .balance
                    .as_ref()
                    .map(|x| as_syntax_amount(x, opts).into()),
                ..syntax::Posting::new_untracked(src_account)
            });
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        let metadata = self
            .comments
            .iter()
            .map(|x| syntax::Metadata::Comment(Cow::Borrowed(x)))
            .collect();
        Ok(syntax::Transaction {
            effective_date: self.effective_date,
            clear_state: syntax::ClearState::Cleared,
            code: self.code.as_deref().map(Into::into),
            posts,
            metadata,
            ..syntax::Transaction::new(self.date, &self.payee)
        })
    }
}

/// Pair of commodity, used for rate computation.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CommodityPair {
    pub source: String,
    pub target: String,
}

fn as_syntax_amount<'a, T>(amount: T, opts: &'a Options) -> syntax::expr::Amount<'a>
where
    T: AmountRef<'a>,
{
    let amount = amount.into_borrowed();
    let commodity = Cow::Borrowed(
        opts.commodity_rename
            .get(amount.commodity)
            .map(String::as_str)
            .unwrap_or(amount.commodity),
    );
    syntax::expr::Amount {
        // TODO: pass the right format.
        value: PrettyDecimal::unformatted(amount.value),
        commodity,
    }
}

fn amount_with_sign(amount: &'_ OwnedAmount, sign: Decimal) -> BorrowedAmount<'_> {
    let mut ret = amount.into_borrowed();
    ret.value.set_sign_positive(sign.is_sign_positive());
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
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

    fn syntax_amount(value: PrettyDecimal, commodity: &'_ str) -> syntax::expr::ValueExpr<'_> {
        syntax::expr::Amount {
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
            txn.dest_amount(&Options::default()),
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
            txn.dest_amount(&Options::default()),
            syntax::PostingAmount {
                amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                cost: Some(syntax::Exchange::Rate(syntax_amount(
                    PrettyDecimal::unformatted(dec!(100)),
                    "JPY"
                ))),
                lot: syntax::Lot::default(),
            },
        )
    }

    #[test]
    fn dest_amount_aliased() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "日本円"),
        );
        txn.add_rate(
            CommodityPair {
                source: "日本円".to_owned(),
                target: "米ドル".to_owned(),
            },
            dec!(100),
        )
        .unwrap();
        txn.transferred_amount(owned_amount(dec!(10.00), "米ドル"));
        let options = Options {
            commodity_rename: hashmap! {
                "米ドル".to_string() => "USD".to_string(),
                "日本円".to_string() => "JPY".to_string(),
            },
        };

        assert_eq!(
            txn.dest_amount(&options),
            syntax::PostingAmount {
                amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                cost: Some(syntax::Exchange::Rate(syntax_amount(
                    PrettyDecimal::unformatted(dec!(100)),
                    "JPY"
                ))),
                lot: syntax::Lot::default(),
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
            txn.dest_amount(&Options::default()),
            syntax::PostingAmount {
                amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                cost: Some(syntax::Exchange::Rate(syntax_amount(
                    PrettyDecimal::unformatted(dec!(100)),
                    "JPY"
                ))),
                lot: syntax::Lot::default(),
            },
        )
    }
}
