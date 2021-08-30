use rust_decimal::Decimal;

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
}

#[derive(Debug, PartialEq)]
pub struct Amount {
    pub value: Decimal,
    pub commodity: String,
}
