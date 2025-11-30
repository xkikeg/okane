use std::{borrow::Cow, collections::HashMap};

use chrono::NaiveDate;
use pretty_decimal::PrettyDecimal;
use rust_decimal::Decimal;

use okane_core::{
    report::{self, ReportContext},
    syntax,
    utility::ConfigResolver,
};

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

    /// Asserted balance after the transaction.
    balance: Option<OwnedAmount>,

    /// List of charges incurred for the transaction.
    charges: Vec<Charge>,
}

/// Optional object to control the output (such as commodity renaming).
#[derive(Debug, Default)]
pub struct Options {
    /// Renames the commodity in the key into the corresponding value.
    pub commodity_rename: HashMap<String, String>,

    /// Defines the commodity format
    pub commodity_format: ConfigResolver<String, syntax::display::CommodityDisplayOption>,
}

/// Pair of commodity, used for rate computation.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CommodityPair {
    pub source: String,
    pub target: String,
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
const LABEL_ADJUSTMENTS: &str = "Equity:Adjustments";
const LABEL_UNKNOWN_INCOME: &str = "Income:Unknown";
const LABEL_UNKNOWN_EXPENSE: &str = "Expenses:Unknown";

impl Txn {
    /// Create a new single entry [`Txn`].
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

    /// Sets the transferred amount, which might be different from amount.
    /// Typically this is useful for multi-commodity transaction.
    pub fn transferred_amount(&mut self, transferred: OwnedAmount) -> &mut Txn {
        self.transferred_amount = Some(amount_with_sign(transferred, -self.amount.value));
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

    /// Sets the balance after transaction.
    pub fn balance(&mut self, balance: OwnedAmount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    fn rate<'a>(&'a self, target: &str) -> Option<BorrowedAmount<'a>> {
        self.rates.get(target).map(|x| x.as_borrowed())
    }

    /// Converts the given amount into [`PostingAmount`].
    fn new_posting_amount<'a>(&'a self, amount: BorrowedAmount<'a>) -> PostingAmount<'a> {
        PostingAmount {
            amount,
            rate: self.rate(amount.commodity),
        }
    }

    fn dest_amount<'a>(&'a self) -> PostingAmount<'a> {
        let a = self
            .transferred_amount
            .as_ref()
            .map(|transferred| transferred.as_borrowed())
            .unwrap_or_else(|| -self.amount.as_borrowed());
        self.new_posting_amount(a)
    }

    pub fn to_double_entry<'a>(
        &'a self,
        src_account: &'a str,
        opts: &'a Options,
        ctx: &mut ReportContext<'a>,
    ) -> Result<syntax::plain::Transaction<'a>, ImportError> {
        // technically we don't need to store this intermediate Posting
        // into Vec but just use iterator. However, we're not that constrainted
        // to save that extra heap allocation after all.
        let mut posts: Vec<Posting> = Vec::new();
        let post_clear = self.clear_state.unwrap_or(match &self.dest_account {
            Some(_) => syntax::ClearState::Uncleared,
            None => syntax::ClearState::Pending,
        });
        let add_charges = |posts: &mut Vec<Posting<'a>>| {
            for chrg in &self.charges {
                posts.push(Posting {
                    account: LABEL_COMMISSIONS,
                    clear_state: syntax::ClearState::Uncleared,
                    amount: self.new_posting_amount(chrg.amount.as_borrowed()),
                    balance: None,
                    metadata: vec![syntax::Metadata::KeyValueTag {
                        key: Cow::Borrowed("Payee"),
                        value: syntax::MetadataValue::Text(chrg.payee.as_str().into()),
                    }],
                });
            }
        };
        let main_amount = self.new_posting_amount(self.amount.as_borrowed());
        if self.amount.value.is_sign_positive() {
            posts.push(Posting {
                account: src_account,
                clear_state: syntax::ClearState::Uncleared,
                amount: main_amount,
                balance: self.balance.as_ref().map(|x| x.as_borrowed()),
                metadata: Vec::new(),
            });
            add_charges(&mut posts);
            posts.push(Posting {
                account: self.dest_account.as_deref().unwrap_or(LABEL_UNKNOWN_INCOME),
                clear_state: post_clear,
                amount: self.dest_amount(),
                balance: None,
                metadata: Vec::new(),
            });
        } else if self.amount.value.is_sign_negative() {
            posts.push(Posting {
                account: self
                    .dest_account
                    .as_deref()
                    .unwrap_or(LABEL_UNKNOWN_EXPENSE),
                clear_state: post_clear,
                amount: self.dest_amount(),
                balance: None,
                metadata: Vec::new(),
            });
            add_charges(&mut posts);
            posts.push(Posting {
                account: src_account,
                clear_state: syntax::ClearState::Uncleared,
                amount: main_amount,
                balance: self.balance.as_ref().map(|x| x.as_borrowed()),
                metadata: Vec::new(),
            });
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        if let Some(excess) = posting_excess(ctx, &posts, opts)? {
            posts.push(Posting {
                account: LABEL_ADJUSTMENTS,
                clear_state: syntax::ClearState::Pending,
                amount: self.new_posting_amount(excess),
                balance: None,
                metadata: Vec::new(),
            });
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
            posts: posts.into_iter().map(|p| p.as_syntax(opts)).collect(),
            metadata,
            ..syntax::Transaction::new(self.date, &self.payee)
        })
    }
}

/// Balances the given posting.
/// Similar to logic in core/src/report/book_keeping.rs,
/// But we simplifies a lot because of simpler condition.
fn posting_excess<'a>(
    ctx: &mut ReportContext<'a>,
    postings: &[Posting<'a>],
    opts: &Options,
) -> Result<Option<BorrowedAmount<'a>>, ImportError> {
    let mut total = report::Amount::zero();
    for posting in postings {
        total += posting.amount.contributing_amount(ctx);
    }
    if total.maybe_pair().is_some() {
        // given the transaction has 2 commodities without rate,
        // it'll be always balanced.
        return Ok(None);
    }
    let v = total.into_values();
    if v.len() > 1 {
        return Err(ImportError::Other(
            "unbalanced posting because of too many commodities without rate".to_string(),
        ));
    }
    let Some((c, excess)) = v.into_iter().next() else {
        return Ok(None);
    };
    let Some(commodity) = ctx.commodity_store().get(c) else {
        return Err(ImportError::Other(format!(
            "unknown commodity: {}",
            c.to_owned_lossy(ctx.commodity_store())
        )));
    };
    let scale = opts
        .commodity_format
        .get(commodity.as_str(), |o| o.min_scale)
        .unwrap_or(0);
    let excess = excess.round_dp(scale.into());
    if excess == Decimal::ZERO {
        return Ok(None);
    }
    Ok(Some(BorrowedAmount {
        value: -excess,
        commodity: commodity.as_str(),
    }))
}

/// Simple posting, which can be converted to [`syntax::plain::Posting`] later.
#[derive(Debug, PartialEq, Eq)]
struct Posting<'a> {
    account: &'a str,
    amount: PostingAmount<'a>,
    balance: Option<BorrowedAmount<'a>>,
    clear_state: syntax::ClearState,
    metadata: Vec<syntax::Metadata<'a>>,
}

/// Simple PostingAmount, which can be converted into [`syntax::plain::PostingAmount`] later.
#[derive(Debug, PartialEq, Eq)]
struct PostingAmount<'a> {
    amount: BorrowedAmount<'a>,
    /// Rate of the posting, eg @ part in 1234 USD @ 123.4 JPY
    rate: Option<BorrowedAmount<'a>>,
}

impl<'a> PostingAmount<'a> {
    /// Computes the amount contributing to the posting balance.
    fn contributing_amount<'ctx>(
        &self,
        ctx: &mut report::ReportContext<'ctx>,
    ) -> report::SingleAmount<'ctx> {
        let (v, c) = match self.rate {
            None => (self.amount.value, self.amount.commodity),
            Some(rate) => (self.amount.value * rate.value, rate.commodity),
        };
        report::SingleAmount::from_value(v, ctx.commodity_store_mut().ensure(c))
    }

    /// Gets the syntax version of posting.
    fn as_syntax(self, opts: &'a Options) -> syntax::plain::PostingAmount<'a> {
        syntax::PostingAmount {
            amount: as_syntax_amount(self.amount, opts).into(),
            cost: self
                .rate
                .map(|x| syntax::Exchange::Rate(as_syntax_amount(x, opts).into())),
            lot: syntax::Lot::default(),
        }
    }
}

impl<'a> Posting<'a> {
    /// Gets the syntax version of posting.
    fn as_syntax(self, opts: &'a Options) -> syntax::plain::Posting<'a> {
        syntax::Posting {
            clear_state: self.clear_state,
            amount: Some(self.amount.as_syntax(opts)),
            balance: self.balance.map(|x| as_syntax_amount(x, opts).into()),
            metadata: self.metadata,
            ..syntax::Posting::new_untracked(self.account)
        }
    }
}

fn as_syntax_amount<'a, T>(amount: T, opts: &'a Options) -> syntax::expr::Amount<'a>
where
    T: AmountRef<'a>,
{
    let amount = amount.as_borrowed();
    let commodity = Cow::Borrowed(
        opts.commodity_rename
            .get(amount.commodity)
            .map(String::as_str)
            .unwrap_or(amount.commodity),
    );
    syntax::expr::Amount {
        // This amount is reformatted with DisplayContext at last.
        value: PrettyDecimal::unformatted(amount.value),
        commodity,
    }
}

fn amount_with_sign(mut amount: OwnedAmount, sign: Decimal) -> OwnedAmount {
    amount.value.set_sign_positive(sign.is_sign_positive());
    amount
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

    fn borrowed_amount(value: Decimal, commodity: &'_ str) -> BorrowedAmount<'_> {
        BorrowedAmount { value, commodity }
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
            txn.dest_amount(),
            PostingAmount {
                amount: borrowed_amount(dec!(-10), "JPY"),
                rate: None
            }
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
            PostingAmount {
                amount: borrowed_amount(dec!(-10.00), "USD"),
                rate: Some(borrowed_amount(dec!(100), "JPY")),
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
            PostingAmount {
                amount: borrowed_amount(dec!(-10.00), "USD"),
                rate: Some(borrowed_amount(dec!(100), "JPY")),
            },
        )
    }

    mod posting_amount {
        use super::*;

        use pretty_assertions::assert_eq;

        #[test]
        fn as_syntax_with_rate() {
            let amount = PostingAmount {
                amount: borrowed_amount(dec!(-10.00), "USD"),
                rate: Some(borrowed_amount(dec!(100), "JPY")),
            };

            assert_eq!(
                amount.as_syntax(&Options::default()),
                syntax::PostingAmount {
                    amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                    cost: Some(syntax::Exchange::Rate(syntax_amount(
                        PrettyDecimal::unformatted(dec!(100)),
                        "JPY"
                    ))),
                    lot: syntax::Lot::default(),
                },
            );
        }

        #[test]
        fn as_syntax_with_alias() {
            let amount = PostingAmount {
                amount: borrowed_amount(dec!(-10.00), "米ドル"),
                rate: Some(borrowed_amount(dec!(100), "日本円")),
            };

            let options = Options {
                commodity_rename: hashmap! {
                    "米ドル".to_string() => "USD".to_string(),
                    "日本円".to_string() => "JPY".to_string(),
                },
                commodity_format: ConfigResolver::default(),
            };

            assert_eq!(
                amount.as_syntax(&options),
                syntax::PostingAmount {
                    amount: syntax_amount(PrettyDecimal::unformatted(dec!(-10.00)), "USD"),
                    cost: Some(syntax::Exchange::Rate(syntax_amount(
                        PrettyDecimal::unformatted(dec!(100)),
                        "JPY"
                    ))),
                    lot: syntax::Lot::default(),
                },
            );
        }
    }
}
