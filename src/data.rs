use std::cmp::Ordering;
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
    pub amount: Amount,
    pub balance: Option<Amount>,
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

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.date.format("%Y/%m/%d"))?;
        if let Some(edate) = &self.effective_date {
            write!(f, "={}", edate.format("%Y/%m/%d"))?;
        }
        write!(f, " {}", print_clear_state(self.clear_state))?;
        if let Some(code) = &self.code {
            write!(f, "({}) ", code)?;
        }
        writeln!(f, "{}", self.payee)?;
        for post in &self.posts {
            let post_clear = print_clear_state(post.clear_state);
            write!(
                f,
                "    {}{}{:>width$} {}",
                post_clear,
                post.account,
                post.amount.value,
                post.amount.commodity,
                width = 48 - post.account.len() - post_clear.len()
            )?;
            if let Some(balance) = &post.balance {
                write!(f, " = {} {}", balance.value, balance.commodity)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

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
            format!("{}", txn)
        );
    }
}
