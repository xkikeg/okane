use rust_decimal::Decimal;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub date: chrono::NaiveDate,
    pub payee: String,
    pub posts: Vec<Post>,
}

#[derive(Debug, PartialEq)]
pub struct Post {
    pub account: String,
    pub amount: Amount,
    pub balance: Option<Amount>,
}

#[derive(Debug, PartialEq)]
pub struct Amount {
    pub value: Decimal,
    pub commodity: String,
}

pub fn parse_comma_decimal(x: &str) -> Result<Decimal, rust_decimal::Error> {
    x.replace(',', "").parse()
}


impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {}", self.date.format("%Y/%m/%d"), self.payee)?;
        for post in &self.posts {
            writeln!(f, "    {}{:>width$} {}", post.account, post.amount.value, post.amount.commodity, width = 48 - post.account.len())?;
        }
        return Ok(());
    }
}
