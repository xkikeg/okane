use std::{borrow::Cow, collections::BTreeMap, collections::HashMap};

use chrono::NaiveDate;
use pretty_decimal::PrettyDecimal;
use rust_decimal::Decimal;

use okane_core::{
    report::{self, ReportContext},
    syntax,
    utility::ConfigResolver,
};

use super::amount::{AmountRef, BorrowedAmount, OwnedAmount};
use super::config::{HiddenFee, HiddenFeeCondition, HiddenFeeRate};
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
    rates: RateRepository,

    hidden_fee: HiddenFee,

    /// Asserted balance after the transaction.
    balance: Option<OwnedAmount>,

    /// List of charges incurred for the transaction.
    charges: Vec<OwnedAmount>,
}

/// Optional object to control the output (such as commodity renaming).
#[derive(Debug, Default)]
pub struct Options {
    /// Operator of the account. Mandatory when charges are needed.
    pub operator: Option<String>,

    /// Renames the commodity in the key into the corresponding value.
    pub commodity_rename: HashMap<String, String>,

    /// Defines the commodity format
    pub commodity_format: ConfigResolver<String, syntax::display::CommodityDisplayOption>,
}

impl Options {
    /// Scale of the corresponding commodity.
    fn scale(&self, commodity: &str) -> Option<u32> {
        self.commodity_format
            .get(commodity, |o| o.min_scale)
            .map(Into::into)
    }

    /// Rounds the given amount to comply with the options.
    fn round<'a>(&self, amount: BorrowedAmount<'a>) -> BorrowedAmount<'a> {
        let Some(scale) = self.scale(amount.commodity) else {
            // no scale found, ok to return as-is.
            return amount;
        };
        BorrowedAmount {
            value: amount.value.round_dp(scale),
            commodity: amount.commodity,
        }
    }
}

/// Pair of commodity, used for rate computation.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CommodityPair {
    pub source: String,
    pub target: String,
}

// TODO: Allow injecting these values from config.
// https://github.com/xkikeg/okane/issues/287
const LABEL_COMMISSIONS: &str = "Expenses:Commissions";
pub(super) const LABEL_ADJUSTMENTS: &str = "Equity:Adjustments";
const LABEL_UNKNOWN_INCOME: &str = "Income:Unknown";
const LABEL_UNKNOWN_EXPENSE: &str = "Expenses:Unknown";

impl Txn {
    /// Create a new single entry [`Txn`].
    /// For those who owns [`super::extract::Fragment`], use [`super::extract::Fragment::new_txn()`].
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
            rates: RateRepository::new(),
            hidden_fee: HiddenFee::default(),
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
        self.rates.add(key, rate)?;
        Ok(self)
    }

    pub fn hidden_fee(&mut self, hidden_fee: HiddenFee) -> &mut Txn {
        self.hidden_fee = hidden_fee;
        self
    }

    pub fn try_add_charge_not_included(
        &mut self,
        amount: OwnedAmount,
    ) -> Result<&mut Txn, ImportError> {
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
        self.charges.push(amount);
        Ok(self)
    }

    pub fn add_charge(&mut self, amount: OwnedAmount) -> &mut Txn {
        self.charges.push(amount);
        self
    }

    /// Sets the balance after transaction.
    pub fn balance(&mut self, balance: OwnedAmount) -> &mut Txn {
        self.balance = Some(balance);
        self
    }

    /// Converts the given amount into [`PostingAmount`].
    fn new_posting_amount<'a>(
        &'a self,
        amount: BorrowedAmount<'a>,
        opts: &Options,
        original_fee: &mut BTreeMap<RateKey<'a>, Decimal>,
    ) -> Result<(PostingAmount<'a>, Option<PostingAmount<'a>>), ImportError> {
        let (main, hidden_fee) = PostingAmount {
            amount,
            rate: self.rates.get(amount.commodity),
        }
        .with_hidden_fee(&self.hidden_fee, opts)?;
        match hidden_fee {
            None => Ok((main, None)),
            Some((rate_key, rate_value, hidden)) => {
                original_fee.insert(rate_key, rate_value);
                Ok((main, Some(hidden)))
            }
        }
    }

    fn dest_amount<'a>(
        &'a self,
        opts: &Options,
        original_fee: &mut BTreeMap<RateKey<'a>, Decimal>,
    ) -> Result<(PostingAmount<'a>, Option<PostingAmount<'a>>), ImportError> {
        let a = self
            .transferred_amount
            .as_ref()
            .map(|transferred| transferred.to_borrowed())
            .unwrap_or_else(|| -self.amount.to_borrowed());
        self.new_posting_amount(a, opts, original_fee)
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
        let mut original_rates = BTreeMap::new();
        // here we store charge amounts into vec, as we may add some fee
        // because of hidden fee.
        let mut charge_amounts: Vec<PostingAmount<'_>> = Vec::new();
        for chrg in &self.charges {
            let (amount, hidden) =
                self.new_posting_amount(chrg.to_borrowed(), opts, &mut original_rates)?;
            charge_amounts.push(amount);
            if let Some(hidden) = hidden {
                charge_amounts.push(hidden);
            }
        }
        let charge_posting = |amount: PostingAmount<'a>| {
            let payee = opts.operator.as_ref().ok_or(ImportError::InvalidConfig(
                "config should have operator to have charge",
            ))?;
            Ok::<_, ImportError>(Posting {
                account: LABEL_COMMISSIONS,
                clear_state: syntax::ClearState::Uncleared,
                amount,
                balance: None,
                metadata: vec![syntax::Metadata::KeyValueTag {
                    key: Cow::Borrowed("Payee"),
                    value: syntax::MetadataValue::Text(payee.into()),
                }],
            })
        };
        let add_charges = |posts: &mut Vec<Posting<'a>>| {
            for chrg in charge_amounts {
                posts.push(charge_posting(chrg)?);
            }
            Ok::<(), ImportError>(())
        };
        let (main_amount, hidden_main_amount) =
            self.new_posting_amount(self.amount.to_borrowed(), opts, &mut original_rates)?;
        let (dest_amount, hidden_amount) = self.dest_amount(opts, &mut original_rates)?;
        if self.amount.value.is_sign_positive() {
            posts.push(Posting {
                account: src_account,
                clear_state: syntax::ClearState::Uncleared,
                amount: main_amount,
                balance: self.balance.as_ref().map(|x| x.to_borrowed()),
                metadata: Vec::new(),
            });
            if let Some(hidden) = hidden_main_amount {
                posts.push(charge_posting(hidden)?);
            }
            add_charges(&mut posts)?;
            posts.push(Posting {
                account: self.dest_account.as_deref().unwrap_or(LABEL_UNKNOWN_INCOME),
                clear_state: post_clear,
                amount: dest_amount,
                balance: None,
                metadata: Vec::new(),
            });
            if let Some(hidden) = hidden_amount {
                posts.push(charge_posting(hidden)?);
            }
        } else if self.amount.value.is_sign_negative() {
            posts.push(Posting {
                account: self
                    .dest_account
                    .as_deref()
                    .unwrap_or(LABEL_UNKNOWN_EXPENSE),
                clear_state: post_clear,
                amount: dest_amount,
                balance: None,
                metadata: Vec::new(),
            });
            if let Some(hidden) = hidden_amount {
                posts.push(charge_posting(hidden)?);
            }
            add_charges(&mut posts)?;
            posts.push(Posting {
                account: src_account,
                clear_state: syntax::ClearState::Uncleared,
                amount: main_amount,
                balance: self.balance.as_ref().map(|x| x.to_borrowed()),
                metadata: Vec::new(),
            });
            if let Some(hidden) = hidden_main_amount {
                posts.push(charge_posting(hidden)?);
            }
        } else {
            // warning log or error?
            return Err(ImportError::Other("credit and debit both zero".to_string()));
        }
        if let Some(excess) = posting_excess(ctx, &posts, opts)? {
            posts.push(Posting {
                account: LABEL_ADJUSTMENTS,
                clear_state: syntax::ClearState::Pending,
                amount: PostingAmount {
                    amount: excess,
                    rate: None,
                },
                balance: None,
                metadata: Vec::new(),
            });
        }
        let metadata = self
            .comments
            .iter()
            .map(|x| syntax::Metadata::Comment(Cow::Borrowed(x)))
            .chain(
                original_rates
                    .iter()
                    .map(|(key, rate)| syntax::Metadata::KeyValueTag {
                        key: Cow::Borrowed("original_rate"),
                        value: syntax::MetadataValue::Text(Cow::Owned(format!(
                            "{} {}/{}",
                            rate, key.target, key.base
                        ))),
                    }),
            )
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
    let excess = opts.round(BorrowedAmount {
        value: -excess,
        commodity: commodity.as_str(),
    });
    if excess.value.is_zero() {
        return Ok(None);
    }
    Ok(Some(excess))
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

/// Rate key, mainly to sort them programatically.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RateKey<'a> {
    target: &'a str,
    base: &'a str,
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

    /// Consider hidden fee, and returns modified amount with the optional commission amount if exists.
    fn with_hidden_fee(
        self,
        hidden_fee: &HiddenFee,
        opts: &Options,
    ) -> Result<(Self, Option<(RateKey<'a>, Decimal, Self)>), ImportError> {
        // hidden fee isn't related when no commodity conversion applied.
        let Some(original_rate) = self.rate else {
            return Ok((self, None));
        };
        // hidden fee isn't configured.
        let Some(hidden_rate) = &hidden_fee.spread else {
            return Ok((self, None));
        };
        let condition = hidden_fee.condition.unwrap_or_default();
        let real_rate_bigger = match condition {
            // Example:
            //   Assets    -5 USD @ 110 JPY
            //   Assets   550 JPY
            // If the bank spread is 10 JPY,
            // the original rate is 120 JPY,
            // and diff 5 * 10 is the commission.
            HiddenFeeCondition::AlwaysIncurred if self.amount.value.is_sign_negative() => true,
            // this case opposite to the above.
            //   Assets     5 USD @ 110 JPY
            //   Assets  -550 JPY
            // If the bank spread is 10 JPY,
            // the original rate is 100 JPY,
            // and diff 5 * 10 is the commission.
            HiddenFeeCondition::AlwaysIncurred => false,
            // debit only always assume rated posting is expense.
            // so it's the same as above.
            // For credit card it makes sense as 99% of 'income' on the credit card
            // is the reimbursement, and the rate won't be computed like AlwaysIncurred.
            HiddenFeeCondition::DebitOnly => false,
        };
        let rate = detach_hidden_rate(original_rate, hidden_rate, real_rate_bigger, opts)?;
        let diff = opts.round(BorrowedAmount {
            value: self.amount.value * (original_rate.value - rate.value),
            commodity: rate.commodity,
        });
        Ok((
            PostingAmount {
                amount: self.amount,
                rate: Some(rate),
            },
            Some((
                RateKey {
                    target: original_rate.commodity,
                    base: self.amount.commodity,
                },
                original_rate.value,
                PostingAmount {
                    amount: diff,
                    rate: None,
                },
            )),
        ))
    }
}

fn detach_hidden_rate<'a>(
    original_rate: BorrowedAmount<'a>,
    hidden_rate: &HiddenFeeRate,
    real_rate_bigger: bool,
    opts: &Options,
) -> Result<BorrowedAmount<'a>, ImportError> {
    let hundred: Decimal = 100.into();
    let mut value = match hidden_rate {
        HiddenFeeRate::Percent(pct) => {
            if real_rate_bigger {
                original_rate.value * hundred / (hundred - pct)
            } else {
                original_rate.value * hundred / (hundred + pct)
            }
        }
        HiddenFeeRate::Fixed(fixed) => {
            if fixed.commodity != original_rate.commodity {
                return Err(ImportError::Other(format!("hidden fee rate commodity {} must match against the transaciton rate commodity {}", fixed.commodity, original_rate.commodity)));
            }
            if real_rate_bigger {
                original_rate.value + fixed.value
            } else {
                original_rate.value - fixed.value
            }
        }
    };
    let scale = match opts.scale(original_rate.commodity) {
        Some(s) => std::cmp::max(original_rate.value.scale(), s),
        None => original_rate.value.scale(),
    };
    value.rescale(scale);
    Ok(BorrowedAmount {
        value,
        commodity: original_rate.commodity,
    })
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
    let amount = amount.to_borrowed();
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

/// Stores rate information.
#[derive(Debug, Clone)]
struct RateRepository {
    /// Rates from commodity to amount, such as USD => 120 JPY.
    rates: HashMap<String, OwnedAmount>,
}

impl RateRepository {
    fn new() -> Self {
        Self {
            rates: HashMap::new(),
        }
    }

    fn get<'a>(&'a self, target: &str) -> Option<BorrowedAmount<'a>> {
        self.rates.get(target).map(|x| x.to_borrowed())
    }

    fn add(&mut self, key: CommodityPair, rate: Decimal) -> Result<(), ImportError> {
        if key.source == key.target {
            return Err(ImportError::Other(format!(
                "cannot handle rate with the same commodity {}",
                key.source
            )));
        }
        let Some(existing) = self.rates.insert(
            key.target.clone(),
            OwnedAmount {
                value: rate,
                commodity: key.source.clone(),
            },
        ) else {
            return Ok(());
        };
        if (&existing.commodity, existing.value) != (&key.source, rate) {
            return Err(ImportError::Other(format!(
                "given commodity {} has two distinct rates: @ {} {} and @ {} {}",
                key.target, existing.value, existing.commodity, key.source, rate
            )));
        }
        Ok(())
    }
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
        let mut original_rates = BTreeMap::new();

        assert_eq!(
            txn.dest_amount(&Options::default(), &mut original_rates)
                .unwrap(),
            (
                PostingAmount {
                    amount: borrowed_amount(dec!(-10), "JPY"),
                    rate: None
                },
                None
            )
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
        let mut original_rates = BTreeMap::new();

        assert_eq!(
            txn.dest_amount(&Options::default(), &mut original_rates)
                .unwrap(),
            (
                PostingAmount {
                    amount: borrowed_amount(dec!(-10.00), "USD"),
                    rate: Some(borrowed_amount(dec!(100), "JPY")),
                },
                None
            )
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
        let mut original_rates = BTreeMap::new();

        assert_eq!(
            txn.dest_amount(&Options::default(), &mut original_rates)
                .unwrap(),
            (
                PostingAmount {
                    amount: borrowed_amount(dec!(-10.00), "USD"),
                    rate: Some(borrowed_amount(dec!(100), "JPY")),
                },
                None
            )
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
                operator: None,
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
