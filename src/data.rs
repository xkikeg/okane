extern crate chrono;
extern crate rust_decimal;

use rust_decimal::Decimal;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    date: chrono::NaiveDate,
    payee: String,
    posts: Vec<Post>,
}

#[derive(Debug, PartialEq)]
pub struct Post {
    account: String,
    amount: Amount,
}

#[derive(Debug, PartialEq)]
pub struct Amount {
    value: Decimal,
    commodity: String,
}
