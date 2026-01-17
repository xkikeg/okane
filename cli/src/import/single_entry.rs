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
use super::config::{HiddenFee, HiddenFeeCondition, HiddenFeeRate};
use super::error::{ImportError, ImportErrorKind, IntoImportError};

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
    rate: Option<Rate>,

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
    /// Returns commodity name.
    fn commodity<'a>(&'a self, commodity: &'a str) -> &'a str {
        self.commodity_rename
            .get(commodity)
            .map(String::as_str)
            .unwrap_or(commodity)
    }

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
            rate: None,
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

    /// Adds the commodity rate information.
    ///
    /// 1 target == rate.
    pub fn rate(&mut self, target: String, rate: OwnedAmount) -> &mut Txn {
        self.rate = Some(Rate { target, rate });
        self
    }

    pub fn hidden_fee(&mut self, hidden_fee: HiddenFee) -> &mut Txn {
        self.hidden_fee = hidden_fee;
        self
    }

    /// Add charge that is not a part of transferred_amount.
    pub fn try_add_charge_not_included(
        &mut self,
        amount: OwnedAmount,
    ) -> Result<&mut Txn, ImportError> {
        if amount.commodity != self.amount.commodity {
            return Err(ImportError::new(
                ImportErrorKind::Unimplemented,
                "non-inclusive charge with different commodity not supported".to_string(),
            ));
        }
        if self.transferred_amount.is_some() {
            return Err(ImportError::new(
                ImportErrorKind::Unimplemented,
                "non-inclusive charge with already set transferred_amount isn't supported"
                    .to_string(),
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

    fn dest_amount<'a>(&'a self) -> BorrowedAmount<'a> {
        self.transferred_amount
            .as_ref()
            .map(|transferred| transferred.to_borrowed())
            .unwrap_or_else(|| -self.amount.to_borrowed())
    }

    pub fn to_double_entry<'a>(
        &'a self,
        src_account: &'a str,
        opts: &'a Options,
        ctx: &mut ReportContext<'a>,
    ) -> Result<syntax::plain::Transaction<'a>, ImportError> {
        let mut postings: Vec<PostingWithoutRate<'a>> = Vec::new();
        let charges = || self.charges.iter().map(<&OwnedAmount>::to_borrowed);
        let add_charges = |postings: &mut Vec<PostingWithoutRate<'a>>| {
            postings
                .extend(charges().map(|charge| PostingWithoutRate(PostingType::Commission, charge)))
        };
        let dest_account: &str;
        if self.amount.value.is_sign_positive() {
            postings.push(PostingWithoutRate(
                PostingType::Main,
                self.amount.to_borrowed(),
            ));
            add_charges(&mut postings);
            postings.push(PostingWithoutRate(PostingType::Dest, self.dest_amount()));
            dest_account = self.dest_account.as_deref().unwrap_or(LABEL_UNKNOWN_INCOME);
        } else {
            postings.push(PostingWithoutRate(PostingType::Dest, self.dest_amount()));
            dest_account = self
                .dest_account
                .as_deref()
                .unwrap_or(LABEL_UNKNOWN_EXPENSE);
            add_charges(&mut postings);
            postings.push(PostingWithoutRate(
                PostingType::Main,
                self.amount.to_borrowed(),
            ));
        }
        let rate: Option<AmendedRate<'_>> = self
            .rate
            .as_ref()
            .map(|r| {
                r.apply_hidden_fee(
                    opts,
                    &self.hidden_fee,
                    // prefer main / dest amount rather than charges.
                    ([self.amount.to_borrowed(), self.dest_amount()])
                        .into_iter()
                        .chain(charges()),
                )
            })
            .transpose()?;
        let mut rated_postings: Vec<Posting<'_>> = Vec::new();
        for p in postings {
            let (converted, hidden) = p.with_rate(rate.as_ref(), opts);
            rated_postings.push(converted);
            if let Some(hidden) = hidden {
                rated_postings.push(hidden);
            }
        }
        if let Some(excess) = posting_excess(ctx, &rated_postings, opts)? {
            rated_postings.push(Posting {
                posting_type: PostingType::Adjustment,
                amount: excess,
            });
        }
        let operator = || {
            opts.operator
                .as_ref()
                .map(|o| Cow::Borrowed(o.as_str()))
                .into_import_err(
                    ImportErrorKind::InvalidConfig,
                    concat!(
                        "config operator field is missing:",
                        " config should have operator to import transactions with charges"
                    ),
                )
        };
        let post_cleared = self.clear_state.unwrap_or(match &self.dest_account {
            Some(_) => syntax::ClearState::Uncleared,
            None => syntax::ClearState::Pending,
        });
        let mut postings = Vec::new();
        for p in rated_postings {
            let sp = match p.posting_type {
                PostingType::Main => syntax::Posting {
                    clear_state: syntax::ClearState::Uncleared,
                    amount: Some(p.amount.into_syntax(opts)),
                    balance: self
                        .balance
                        .as_ref()
                        .map(|x| into_syntax_amount(x.to_borrowed(), opts).into()),
                    metadata: Vec::new(),
                    ..syntax::Posting::new_untracked(src_account)
                },
                PostingType::Dest => syntax::Posting {
                    clear_state: post_cleared,
                    amount: Some(p.amount.into_syntax(opts)),
                    balance: None,
                    metadata: Vec::new(),
                    ..syntax::Posting::new_untracked(dest_account)
                },
                PostingType::Commission => syntax::Posting {
                    clear_state: syntax::ClearState::Uncleared,
                    amount: Some(p.amount.into_syntax(opts)),
                    balance: None,
                    metadata: vec![syntax::Metadata::KeyValueTag {
                        key: Cow::Borrowed("Payee"),
                        value: syntax::MetadataValue::Text(operator()?),
                    }],
                    ..syntax::Posting::new_untracked(LABEL_COMMISSIONS)
                },
                PostingType::Adjustment => syntax::Posting {
                    clear_state: syntax::ClearState::Pending,
                    amount: Some(p.amount.into_syntax(opts)),
                    balance: None,
                    metadata: vec![syntax::Metadata::KeyValueTag {
                        key: Cow::Borrowed("Payee"),
                        value: syntax::MetadataValue::Text(operator()?),
                    }],
                    ..syntax::Posting::new_untracked(LABEL_ADJUSTMENTS)
                },
            };
            postings.push(sp);
        }
        let mut metadata: Vec<syntax::Metadata<'_>> = self
            .comments
            .iter()
            .map(|x| syntax::Metadata::Comment(Cow::Borrowed(x)))
            .collect();
        if let Some(rate) = &rate {
            if !rate.spread.value.is_zero() {
                metadata.push(syntax::Metadata::KeyValueTag {
                    key: Cow::Borrowed("original_rate"),
                    value: syntax::MetadataValue::Text(Cow::Owned(format!(
                        "{} {}/{}",
                        rate.original.value,
                        opts.commodity(rate.original.commodity),
                        opts.commodity(rate.target),
                    ))),
                });
            }
        }
        Ok(syntax::Transaction {
            effective_date: self.effective_date,
            clear_state: syntax::ClearState::Cleared,
            code: self.code.as_deref().map(Into::into),
            posts: postings,
            metadata,
            ..syntax::Transaction::new(self.date, &self.payee)
        })
    }
}

/// Abstracted posting type, which will be converted [`syntax::Posting`] later.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PostingType {
    /// The amount you get or lose.
    Main,
    /// Destionation, the amount on the counter party.
    Dest,
    /// Commission incurred by the operator.
    Commission,
    /// adjustments to balance posting.
    Adjustment,
}

/// Posting without rate, later will be converted into [`Posting`].
#[derive(Debug, PartialEq, Eq)]
struct PostingWithoutRate<'a>(PostingType, BorrowedAmount<'a>);

impl<'a> PostingWithoutRate<'a> {
    fn with_rate(
        self,
        rate: Option<&AmendedRate<'a>>,
        opts: &Options,
    ) -> (Posting<'a>, Option<Posting<'a>>) {
        let Self(posting_type, amount) = self;
        let posting = Posting {
            posting_type,
            amount: PostingAmount { amount, rate: None },
        };
        let Some(rate) = rate else {
            return (posting, None);
        };
        if rate.target != posting.amount.amount.commodity {
            return (posting, None);
        }
        let posting = Posting {
            posting_type,
            amount: PostingAmount {
                amount,
                rate: Some(rate.amended),
            },
        };
        if rate.spread.value.is_zero() {
            return (posting, None);
        }
        let hidden = Posting {
            posting_type: PostingType::Commission,
            amount: PostingAmount {
                amount: opts.round(BorrowedAmount {
                    commodity: rate.amended.commodity,
                    value: amount.value * rate.spread.value,
                }),
                rate: None,
            },
        };
        (posting, Some(hidden))
    }
}

/// Simple posting, which can be converted to [`syntax::plain::Posting`] later.
#[derive(Debug, PartialEq, Eq)]
struct Posting<'a> {
    posting_type: PostingType,
    amount: PostingAmount<'a>,
}

/// Simple PostingAmount, which can be converted into [`syntax::plain::PostingAmount`] later.
#[derive(Debug, PartialEq, Eq)]
struct PostingAmount<'a> {
    amount: BorrowedAmount<'a>,
    /// Rate of the posting, eg @ part in 1234 USD @ 123.4 JPY
    rate: Option<BorrowedAmount<'a>>,
}

/// Balances the given posting.
/// Similar to logic in core/src/report/book_keeping.rs,
/// But we simplifies a lot because of simpler condition.
fn posting_excess<'a>(
    ctx: &mut ReportContext<'a>,
    postings: &[Posting<'a>],
    opts: &Options,
) -> Result<Option<PostingAmount<'a>>, ImportError> {
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
        return Err(ImportError::new(
            ImportErrorKind::Unimplemented,
            format!(
                "impossible to balance posting excess {} due to too many commodities without rate",
                report::Amount::from_values(v).as_inline_display(ctx)
            ),
        ));
    }
    let Some((c, excess)) = v.into_iter().next() else {
        return Ok(None);
    };
    let Some(commodity) = ctx.commodity_store().get(c) else {
        return Err(ImportError::new(
            ImportErrorKind::Internal,
            format!(
                "unknown commodity: {}",
                c.to_owned_lossy(ctx.commodity_store())
            ),
        ));
    };
    let excess = opts.round(BorrowedAmount {
        value: -excess,
        commodity: commodity.as_str(),
    });
    if excess.value.is_zero() {
        return Ok(None);
    }
    Ok(Some(PostingAmount {
        amount: excess,
        rate: None,
    }))
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
        report::SingleAmount::from_value(ctx.commodity_store_mut().ensure(c), v)
    }

    /// Gets the syntax version of posting.
    fn into_syntax(self, opts: &'a Options) -> syntax::plain::PostingAmount<'a> {
        syntax::PostingAmount {
            amount: into_syntax_amount(self.amount, opts).into(),
            cost: self
                .rate
                .map(|x| syntax::Exchange::Rate(into_syntax_amount(x, opts).into())),
            lot: syntax::Lot::default(),
        }
    }
}

fn into_syntax_amount<'a, T>(amount: T, opts: &'a Options) -> syntax::expr::Amount<'a>
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
struct Rate {
    target: String,
    rate: OwnedAmount,
}

/// Amended rate information by hidden fee.
#[derive(Debug)]
struct AmendedRate<'a> {
    target: &'a str,
    original: BorrowedAmount<'a>,
    amended: BorrowedAmount<'a>,
    /// if spread is zero, it means hidden fee was zero.
    /// spread * amount will be the commission.
    spread: BorrowedAmount<'a>,
}

impl Rate {
    fn apply_hidden_fee<'a, T: Iterator<Item = BorrowedAmount<'a>>>(
        &'a self,
        opts: &'a Options,
        hidden_fee: &HiddenFee,
        mut amount: T,
    ) -> Result<AmendedRate<'a>, ImportError> {
        let fallback = AmendedRate {
            target: &self.target,
            original: self.rate.to_borrowed(),
            amended: self.rate.to_borrowed(),
            spread: BorrowedAmount {
                value: Decimal::ZERO,
                commodity: "unused",
            },
        };
        // hidden fee isn't configured.
        let Some(hidden_rate) = &hidden_fee.spread else {
            return Ok(fallback);
        };
        // hidden fee isn't related when no commodity conversion applied.
        let amount = amount.find(|x| x.commodity == self.target);
        log::trace!(
            "apply_hidden_fee target: rate: {self:?} hidden_fee: {hidden_fee:?} amount: {amount:?}"
        );
        let Some(amount) = amount else {
            return Ok(fallback);
        };
        let condition = hidden_fee.condition.unwrap_or_default();
        let real_rate_bigger = match condition {
            // Example:
            //   Assets    -5 USD @ 110 JPY
            //   Assets   550 JPY
            // If the bank spread is 10 JPY,
            // the original rate is 120 JPY,
            // and diff 5 * 10 is the commission.
            HiddenFeeCondition::AlwaysIncurred if amount.value.is_sign_negative() => true,
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
        let rate =
            detach_hidden_rate(self.rate.to_borrowed(), hidden_rate, real_rate_bigger, opts)?;
        let spread = BorrowedAmount {
            value: self.rate.value - rate.value,
            commodity: rate.commodity,
        };
        let ret = AmendedRate {
            target: &self.target,
            original: self.rate.to_borrowed(),
            amended: rate,
            spread,
        };
        log::trace!("amended rate: {ret:?}");
        Ok(ret)
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
                return Err(ImportError::new(
                    ImportErrorKind::InvalidConfig,
                    format!(
                        "hidden fee rate commodity {} must match against the transaciton rate commodity {}",
                        fixed.commodity, original_rate.commodity
                    ),
                ));
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
    fn try_add_charge_not_included_fails_on_different_commodity() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );

        let got_err = txn
            .try_add_charge_not_included(owned_amount(dec!(1), "CHF"))
            .unwrap_err();

        assert_eq!(ImportErrorKind::Unimplemented, got_err.error_kind());
        assert_eq!(
            "non-inclusive charge with different commodity not supported",
            got_err.message()
        );
    }

    #[test]
    fn try_add_charge_not_included_fails_on_already_transferred_amount_set_txn() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );

        let got_err = txn
            .transferred_amount(owned_amount(dec!(900), "JPY"))
            .try_add_charge_not_included(owned_amount(dec!(100), "JPY"))
            .unwrap_err();

        assert_eq!(ImportErrorKind::Unimplemented, got_err.error_kind());
        assert_eq!(
            "non-inclusive charge with already set transferred_amount isn't supported",
            got_err.message()
        );
    }

    #[test]
    fn try_add_charge_not_included_with_negative_amount() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(-1000), "JPY"),
        );

        txn.try_add_charge_not_included(owned_amount(dec!(100), "JPY"))
            .expect("try_add_charge must succeed");

        assert_eq!(vec![owned_amount(dec!(100), "JPY")], txn.charges);
        assert_eq!(borrowed_amount(dec!(900), "JPY"), txn.dest_amount());
    }

    #[test]
    fn try_add_charge_not_included_positive_amount() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );

        txn.try_add_charge_not_included(owned_amount(dec!(100), "JPY"))
            .expect("try_add_charge must succeed");

        assert_eq!(vec![owned_amount(dec!(100), "JPY")], txn.charges);
        assert_eq!(borrowed_amount(dec!(-1100), "JPY"), txn.dest_amount());
    }

    #[test]
    fn dest_amount_plain() {
        let txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(10), "JPY"),
        );

        assert_eq!(txn.dest_amount(), borrowed_amount(dec!(-10), "JPY"));
    }

    #[test]
    fn dest_amount_exchanged() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );
        txn.rate(
            "USD".to_owned(),
            OwnedAmount {
                commodity: "JPY".to_owned(),
                value: dec!(100),
            },
        );
        txn.transferred_amount(owned_amount(dec!(10.00), "USD"));

        assert_eq!(txn.dest_amount(), borrowed_amount(dec!(-10.00), "USD"));
    }

    #[test]
    fn dest_amount_transferred_negative() {
        let mut txn = Txn::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "foo",
            owned_amount(dec!(1000), "JPY"),
        );
        txn.rate(
            "USD".to_owned(),
            OwnedAmount {
                commodity: "JPY".to_owned(),
                value: dec!(100),
            },
        );
        txn.transferred_amount(owned_amount(dec!(-10.00), "USD"));

        assert_eq!(txn.dest_amount(), borrowed_amount(dec!(-10.00), "USD"));
    }

    mod to_double_entry {
        use super::*;

        use bumpalo::Bump;
        use pretty_assertions::assert_eq;

        #[test]
        fn fails_on_unrated_operator_unset() {
            let arena = Bump::new();
            let mut ctx = ReportContext::new(&arena);
            let opts = Options {
                operator: None,
                commodity_format: ConfigResolver::default(),
                commodity_rename: HashMap::new(),
            };
            let mut txn = Txn::new(
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                "foo",
                owned_amount(dec!(1000), "JPY"),
            );
            txn.add_charge(owned_amount(dec!(10), "JPY"))
                .transferred_amount(owned_amount(dec!(1010), "JPY"));

            let got_err = txn
                .to_double_entry("Assets:Bank", &opts, &mut ctx)
                .unwrap_err();
            assert_eq!(
                ImportErrorKind::InvalidConfig,
                got_err.error_kind(),
                "error={got_err:?}"
            );
            assert_eq!(
                concat!(
                    "config operator field is missing:",
                    " config should have operator to import transactions with charges"
                ),
                got_err.message()
            );
        }

        #[test]
        fn fails_on_unrated_three_or_more_commodities() {
            let arena = Bump::new();
            let mut ctx = ReportContext::new(&arena);
            let opts = Options {
                operator: Some("Okane bank".to_string()),
                commodity_format: ConfigResolver::default(),
                commodity_rename: HashMap::new(),
            };
            let mut txn = Txn::new(
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                "foo",
                owned_amount(dec!(-9000), "JPY"),
            );
            txn.add_charge(owned_amount(dec!(1), "USD"))
                .add_charge(owned_amount(dec!(1), "CHF"))
                .transferred_amount(owned_amount(dec!(8700), "JPY"));

            let got_err = txn
                .to_double_entry("Assets:Bank", &opts, &mut ctx)
                .unwrap_err();
            assert_eq!(
                ImportErrorKind::Unimplemented,
                got_err.error_kind(),
                "error={got_err:?}"
            );
        }
    }

    mod posting_amount {
        use super::*;

        use pretty_assertions::assert_eq;

        #[test]
        fn into_syntax_with_rate() {
            let amount = PostingAmount {
                amount: borrowed_amount(dec!(-10.00), "USD"),
                rate: Some(borrowed_amount(dec!(100), "JPY")),
            };

            assert_eq!(
                amount.into_syntax(&Options::default()),
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
        fn into_syntax_with_alias() {
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
                amount.into_syntax(&options),
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
