//! Computes amount expressions.

use rust_decimal::Decimal;

use crate::{
    eval::{context, types},
    repl::{self, pretty_decimal::PrettyDecimal, LedgerEntry},
};

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Amount<'ctx> {
    values: HashMap<types::Commodity<'ctx>, Decimal>,
}

#[derive(Debug, Default)]
pub struct AccountBalance {
    accounts: HashMap<types::Account<'ctx>, Amount>,
}

impl AccountBalance {
    pub fn add(&mut self, postings: &[repl::Posting]) {
        for posting in postings {
            let balance = self
                .accounts
                .entry(&posting.account)
                .or_insert(Amount::default());
            balance.add
        }
        todo!()
    }
}
