use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use rust_decimal::Decimal;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, PartialEq)]
/// Represents a transaction where the money transfered across the accounts.
pub struct Transaction {
    /// Date when the transaction issued.
    pub date: chrono::NaiveDate,
    /// Date when the transaction got effective, optional.
    pub effective_date: Option<chrono::NaiveDate>,
    /// Indiacates clearing state of the entire transaction.
    pub clear_state: ClearState,
    /// Transaction code (not necessarily unique).
    pub code: Option<String>,
    /// Label of the transaction, often the opposite party of the transaction.
    pub payee: String,
    /// Postings of the transaction, could be empty.
    pub posts: Vec<Post>,
}

impl Transaction {
    /// Constructs minimal transaction.
    pub fn new(date: chrono::NaiveDate, payee: String) -> Transaction {
        Transaction {
            date,
            effective_date: None,
            clear_state: ClearState::Uncleared,
            code: None,
            payee,
            posts: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Represents a clearing state, often combined with the ambiguity.
pub enum ClearState {
    /// No specific meaning.
    Uncleared,
    /// Useful to declare that the transaction / post is confirmed.
    Cleared,
    /// Useful to declare that the transaction / post is still pending.
    Pending,
}

impl Default for ClearState {
    fn default() -> ClearState {
        ClearState::Uncleared
    }
}

#[derive(Debug, PartialEq)]
/// Post is a posting in a transaction, and
/// it represents a particular account increase / decrease.
pub struct Post {
    /// Account of the post target.
    pub account: String,
    /// Posting specific ClearState.
    pub clear_state: ClearState,
    /// Amount of the posting.
    pub amount: Option<ExchangedAmount>,
    /// Balance after the transaction of the specified account.
    pub balance: Option<Amount>,
    /// Overwrites the payee.
    pub payee: Option<String>,
}

impl Post {
    pub fn new(account: String) -> Post {
        Post {
            account,
            clear_state: ClearState::default(),
            amount: None,
            balance: None,
            payee: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Amount with the currency exchange information.
pub struct ExchangedAmount {
    /// Amount of the original value.
    pub amount: Amount,
    /// Exchange rate information.
    pub exchange: Option<Exchange>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Commodity exchange information.
pub enum Exchange {
    /// Represents the amount equals to the `ExchangedAmount.amount`.
    Total(Amount),
    /// Represents te amount equals to 1 `ExchangedAmount.amount.commodity`.
    Rate(Amount),
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Amount of posting, balance, ...
pub struct Amount {
    /// Numerical value.
    pub value: Decimal,
    /// Commodity aka currency.
    pub commodity: String,
}

impl Amount {
    /// Returns `true` if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    /// Returns `true` if the amount is positive.
    pub fn is_sign_positive(&self) -> bool {
        self.value.is_sign_positive()
    }

    /// Returns `true` if the amount is negative.
    pub fn is_sign_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
}

/// # Examples
///
/// ```
/// # use rust_decimal_macros::dec;
/// let x = okane::data::Amount{
///     value: dec!(-5),
///     commodity: "JPY".to_string(),
/// };
/// let y = -x.clone();
/// assert_eq!(x.value, dec!(-5));
/// assert_eq!(x.commodity, "JPY");
/// assert_eq!(y.value, dec!(5));
/// assert_eq!(y.commodity, "JPY");
/// ```
impl std::ops::Neg for Amount {
    type Output = Amount;
    fn neg(self) -> Amount {
        Amount {
            value: -self.value,
            commodity: self.commodity,
        }
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.commodity != other.commodity {
            None
        } else {
            Some(self.value.cmp(&other.value))
        }
    }
}

/// Parses number including comma, returns the decimal.
pub fn parse_comma_decimal(x: &str) -> Result<Decimal, rust_decimal::Error> {
    x.replace(',', "").parse()
}

fn print_clear_state(v: ClearState) -> &'static str {
    match v {
        ClearState::Uncleared => "",
        ClearState::Cleared => "* ",
        ClearState::Pending => "! ",
    }
}

/// Context information to control the formatting of the transaction.
pub struct DisplayContext {
    pub precisions: HashMap<String, u8>,
}

/// Transaction combined with the transaction.
pub struct TransactionWithContext<'a> {
    pub transaction: &'a Transaction,
    pub context: &'a DisplayContext,
}

fn rescale(x: &Amount, context: &DisplayContext) -> Decimal {
    let mut v = x.value;
    v.rescale(std::cmp::max(
        x.value.scale(),
        context.precisions.get(&x.commodity).cloned().unwrap_or(0) as u32,
    ));
    v
}

fn get_column(colsize: usize, left: usize, padding: usize) -> usize {
    if left + padding < colsize {
        colsize - left
    } else {
        padding
    }
}

impl<'a> fmt::Display for TransactionWithContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xact = self.transaction;
        write!(f, "{}", xact.date.format("%Y/%m/%d"))?;
        if let Some(edate) = &xact.effective_date {
            write!(f, "={}", edate.format("%Y/%m/%d"))?;
        }
        write!(f, " {}", print_clear_state(xact.clear_state))?;
        if let Some(code) = &xact.code {
            write!(f, "({}) ", code)?;
        }
        writeln!(f, "{}", xact.payee)?;
        for post in &xact.posts {
            let post_clear = print_clear_state(post.clear_state);
            write!(f, "    {}{}", post_clear, post.account)?;
            let account_width = UnicodeWidthStr::width_cjk(post.account.as_str())
                + UnicodeWidthStr::width(post_clear);
            if let Some(amount) = &post.amount {
                let amount_str = rescale(&amount.amount, self.context).to_string();
                write!(
                    f,
                    "{:>width$}{} {}",
                    "",
                    rescale(&amount.amount, self.context),
                    amount.amount.commodity,
                    width = get_column(
                        48,
                        account_width + UnicodeWidthStr::width(amount_str.as_str()),
                        2
                    )
                )?;
                if let Some(exchange) = &amount.exchange {
                    match exchange {
                        Exchange::Rate(v) => write!(f, " @ {} {}", v.value, v.commodity),
                        Exchange::Total(v) => {
                            write!(f, " @@ {} {}", rescale(v, self.context), v.commodity)
                        }
                    }?
                }
            }
            if let Some(balance) = &post.balance {
                let balance_padding = if post.amount.is_some() {
                    0
                } else {
                    get_column(
                        51 + UnicodeWidthStr::width_cjk(balance.commodity.as_str()),
                        account_width,
                        3,
                    )
                };
                write!(
                    f,
                    "{:>width$} {} {}",
                    " =",
                    rescale(balance, self.context),
                    balance.commodity,
                    width = balance_padding
                )?;
            }
            if let Some(payee) = &post.payee {
                write!(f, "  ; Payee: {}", payee)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Transaction {
    pub fn display<'a>(&'a self, context: &'a DisplayContext) -> TransactionWithContext<'a> {
        TransactionWithContext {
            transaction: self,
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    fn empty_context() -> DisplayContext {
        DisplayContext {
            precisions: hashmap! {},
        }
    }

    #[test]
    fn test_display_effective_date() {
        let txn = Transaction {
            date: NaiveDate::from_ymd(2021, 4, 4),
            effective_date: Some(NaiveDate::from_ymd(2021, 5, 10)),
            clear_state: ClearState::Pending,
            code: Some("#12345".to_string()),
            payee: "Flower shop".to_string(),
            posts: Vec::new(),
        };

        assert_eq!(
            "2021/04/04=2021/05/10 ! (#12345) Flower shop\n",
            format!("{}", txn.display(&empty_context()))
        );
    }

    #[test]
    fn test_display_with_precision() {
        let txn = Transaction {
            date: NaiveDate::from_ymd(2021, 4, 4),
            effective_date: None,
            clear_state: ClearState::Uncleared,
            code: None,
            payee: "FX conversion".to_string(),
            posts: vec![
                Post {
                    account: "Income".to_string(),
                    amount: Some(ExchangedAmount {
                        amount: Amount {
                            commodity: "JPY".to_string(),
                            value: -dec!(1000),
                        },
                        exchange: None,
                    }),
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    account: "Asset".to_string(),
                    amount: Some(ExchangedAmount {
                        amount: Amount {
                            commodity: "USD".to_string(),
                            value: dec!(10),
                        },
                        exchange: Some(Exchange::Total(Amount {
                            commodity: "JPY".to_string(),
                            value: dec!(900),
                        })),
                    }),
                    balance: Some(Amount {
                        commodity: "USD".to_string(),
                        value: dec!(10000),
                    }),
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    account: "ﾃｽｳﾘｮｳ".to_string(),
                    amount: Some(ExchangedAmount {
                        amount: Amount {
                            commodity: "EUR".to_string(),
                            value: dec!(0.1),
                        },
                        exchange: Some(Exchange::Total(Amount {
                            commodity: "USD".to_string(),
                            value: dec!(0.1),
                        })),
                    }),
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: Some("bank x".to_string()),
                },
                Post {
                    account: "手数料など".to_string(),
                    amount: Some(ExchangedAmount {
                        amount: Amount {
                            commodity: "USD".to_string(),
                            value: dec!(0.00123),
                        },
                        exchange: Some(Exchange::Rate(Amount {
                            commodity: "EUR".to_string(),
                            value: dec!(1),
                        })),
                    }),
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    account: "Liability".to_string(),
                    amount: None,
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    account: "Liability 2".to_string(),
                    amount: None,
                    balance: Some(Amount {
                        commodity: "USD".to_string(),
                        value: dec!(-52.34),
                    }),
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    amount: Some(ExchangedAmount {
                        amount: Amount {
                            commodity: "USD".to_string(),
                            value: dec!(1),
                        },
                        exchange: None,
                    }),
                    ..Post::new("Super long long long long long long long long".to_string())
                },
                Post {
                    balance: Some(Amount {
                        commodity: "USD".to_string(),
                        value: dec!(1),
                    }),
                    ..Post::new(
                        "Super long long long long long long long long long long".to_string(),
                    )
                },
            ],
        };
        let context = DisplayContext {
            precisions: hashmap! {"USD".to_string() => 2},
        };
        let want = indoc! {"
        2021/04/04 FX conversion
            Income                                     -1000 JPY
            Asset                                      10.00 USD @@ 900 JPY = 10000.00 USD
            ﾃｽｳﾘｮｳ                                       0.1 EUR @@ 0.10 USD  ; Payee: bank x
            手数料など                               0.00123 USD @ 1 EUR
            Liability
            Liability 2                                          = -52.34 USD
            Super long long long long long long long long  1.00 USD
            Super long long long long long long long long long long  = 1.00 USD
        "};

        assert_eq!(want, format!("{}", txn.display(&context)));
    }
}
