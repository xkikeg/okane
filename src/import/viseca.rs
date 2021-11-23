pub mod format;
pub mod parser;

use super::config;
use super::extract;
use super::single_entry;
use super::ImportError;
use crate::data;

use std::convert::{TryFrom, TryInto};

use regex::Regex;

pub struct VisecaImporter {}

impl super::Importer for VisecaImporter {
    fn import<R: std::io::Read>(
        &self,
        r: &mut R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError> {
        let extractor: extract::Extractor<VisecaMatcher> = (&config.rewrite).try_into()?;
        let mut parser = parser::Parser::new(std::io::BufReader::new(r));
        let mut result = Vec::new();
        while let Some(entry) = parser.parse_entry()? {
            let fragment = extractor.extract(&entry);
            let payee = fragment.payee.unwrap_or_else(|| entry.payee.as_str());
            if fragment.account.is_none() {
                log::warn!("account unmatched at line {}, payee={}", entry.line_count, payee);
            }
            let mut txn = single_entry::Txn::new(
                entry.date,
                payee,
                data::Amount {
                    value: -entry.amount,
                    commodity: config.commodity.clone(),
                },
            );
            txn.effective_date(entry.effective_date)
                .dest_account_option(fragment.account);
            if !fragment.cleared {
                txn.clear_state(data::ClearState::Pending);
            }
            if let Some(exchange) = entry.exchange {
                let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
                    "config should have operator to have charge",
                ))?;
                txn.add_charge(
                    payee,
                    data::Amount {
                        value: -exchange.fee_amount,
                        commodity: exchange.fee_currency,
                    },
                );
                txn.transferred_amount(data::ExchangedAmount {
                    amount: data::Amount {
                        value: -exchange.exchanged_amount,
                        commodity: exchange.exchanged_currency,
                    },
                    exchange: Some(data::Exchange::Rate(data::Amount {
                        value: exchange.rate,
                        commodity: exchange.spent_currency,
                    })),
                });
            }
            result.push(txn);
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy)]
enum Field {
    Payee,
    Category,
}

#[derive(Debug)]
struct VisecaMatcher {
    field: Field,
    pattern: Regex,
}

impl TryFrom<(config::RewriteField, &str)> for VisecaMatcher {
    type Error = ImportError;
    fn try_from((f, v): (config::RewriteField, &str)) -> Result<VisecaMatcher, ImportError> {
        let field = match f {
            config::RewriteField::Payee => Field::Payee,
            config::RewriteField::Category => Field::Category,
            _ => {
                return Err(ImportError::InvalidConfig("unsupported rewrite field"));
            }
        };
        let pattern = Regex::new(v)?;
        Ok(VisecaMatcher { field, pattern })
    }
}

impl<'a> extract::Entity<'a> for VisecaMatcher {
    type T = &'a format::Entry;
}

impl extract::EntityMatcher for VisecaMatcher {
    fn captures<'a>(
        &self,
        fragment: &extract::Fragment<'a>,
        entity: &'a format::Entry,
    ) -> Option<extract::Matched<'a>> {
        let target = match self.field {
            Field::Payee => fragment.payee.unwrap_or_else(|| entity.payee.as_str()),
            Field::Category => entity.category.as_str(),
        };
        self.pattern.captures(target).map(Into::into)
    }
}
