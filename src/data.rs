use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use rust_decimal::Decimal;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub date: chrono::NaiveDate,
    pub effective_date: Option<chrono::NaiveDate>,
    pub clear_state: ClearState,
    pub code: Option<String>,
    pub payee: String,
    pub posts: Vec<Post>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClearState {
    /// No specific meaning.
    Uncleared,
    /// Useful to show the transaction / post is confirmed.
    Cleared,
    /// Useful to show the transaction / post is still pending.
    Pending,
}

impl Default for ClearState {
    fn default() -> ClearState {
        ClearState::Uncleared
    }
}

#[derive(Debug, PartialEq)]
pub struct Post {
    pub account: String,
    pub clear_state: ClearState,
    pub amount: ExchangedAmount,
    pub balance: Option<Amount>,
    pub payee: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExchangedAmount {
    /// Amount of the original value.
    pub amount: Amount,
    /// exchange rate information.
    pub exchange: Option<Exchange>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Exchange {
    /// Total(x) == ExchangedAmount.amount.
    Total(Amount),
    /// Rate(x) == 1 ExchangedAmount.amount.commodity.
    Rate(Amount),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount {
    pub value: Decimal,
    pub commodity: String,
}

impl Amount {
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
    pub fn is_sign_positive(&self) -> bool {
        self.value.is_sign_positive()
    }
    pub fn is_sign_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
}

/// # Examples
///
/// ```
/// use rust_decimal_macros::dec;
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

pub struct DisplayContext {
    pub precisions: HashMap<String, u8>,
}

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
            write!(
                f,
                "    {}{}{:>width$} {}",
                post_clear,
                post.account,
                rescale(&post.amount.amount, self.context),
                post.amount.amount.commodity,
                width = 48 - post.account.len() - post_clear.len()
            )?;
            if let Some(exchange) = &post.amount.exchange {
                match exchange {
                    Exchange::Rate(v) => write!(f, " @ {} {}", v.value, v.commodity),
                    Exchange::Total(v) => {
                        write!(f, " @@ {} {}", rescale(v, self.context), v.commodity)
                    }
                }?
            }
            if let Some(balance) = &post.balance {
                write!(
                    f,
                    " = {} {}",
                    rescale(balance, self.context),
                    balance.commodity
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
                    amount: ExchangedAmount {
                        amount: Amount {
                            commodity: "JPY".to_string(),
                            value: -dec!(1000),
                        },
                        exchange: None,
                    },
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    account: "Asset".to_string(),
                    amount: ExchangedAmount {
                        amount: Amount {
                            commodity: "USD".to_string(),
                            value: dec!(10),
                        },
                        exchange: Some(Exchange::Total(Amount {
                            commodity: "JPY".to_string(),
                            value: dec!(900),
                        })),
                    },
                    balance: Some(Amount {
                        commodity: "USD".to_string(),
                        value: dec!(10000),
                    }),
                    clear_state: ClearState::Uncleared,
                    payee: None,
                },
                Post {
                    account: "Commission".to_string(),
                    amount: ExchangedAmount {
                        amount: Amount {
                            commodity: "EUR".to_string(),
                            value: dec!(0.1),
                        },
                        exchange: Some(Exchange::Total(Amount {
                            commodity: "USD".to_string(),
                            value: dec!(0.1),
                        })),
                    },
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: Some("bank x".to_string()),
                },
                Post {
                    account: "Commission".to_string(),
                    amount: ExchangedAmount {
                        amount: Amount {
                            commodity: "USD".to_string(),
                            value: dec!(0.00123),
                        },
                        exchange: Some(Exchange::Rate(Amount {
                            commodity: "EUR".to_string(),
                            value: dec!(1),
                        })),
                    },
                    balance: None,
                    clear_state: ClearState::Uncleared,
                    payee: None,
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
            Commission                                   0.1 EUR @@ 0.10 USD  ; Payee: bank x
            Commission                               0.00123 USD @ 1 EUR
        "};

        assert_eq!(want, format!("{}", txn.display(&context)));
    }
}
