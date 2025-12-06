pub mod format;
pub mod parser;

use std::convert::{TryFrom, TryInto};

use regex::Regex;

use super::amount::OwnedAmount;
use super::config;
use super::extract;
use super::single_entry;
use super::single_entry::CommodityPair;
use super::ImportError;

pub fn import<R: std::io::Read>(
    r: R,
    config: &config::ConfigEntry,
) -> Result<Vec<single_entry::Txn>, ImportError> {
    let extractor: extract::Extractor<VisecaMatcher> = (&config.rewrite).try_into()?;
    let mut parser =
        parser::Parser::new(std::io::BufReader::new(r), config.commodity.primary.clone());
    let mut result = Vec::new();
    while let Some(entry) = parser.parse_entry()? {
        let line_count = entry.line_count;
        let fragment = extractor.extract(&entry);
        let payee = fragment.payee.unwrap_or(entry.payee.as_str());
        let fragment = extract::Fragment {
            payee: Some(payee),
            ..fragment
        };
        let mut txn = fragment.new_txn(
            entry.date,
            OwnedAmount {
                value: -entry.amount,
                commodity: config.commodity.primary.clone(),
            },
            || format!("line {line_count} payee={payee}"),
        );
        txn.effective_date(entry.effective_date);
        if let Some(exchange) = entry.exchange {
            let spent = entry.spent.ok_or_else(|| {
                ImportError::Viseca(format!(
                    "internal error: exchange should set aside with spent: {}",
                    line_count
                ))
            })?;
            txn.add_rate(
                CommodityPair {
                    source: exchange.equivalent.commodity,
                    target: spent.commodity.clone(),
                },
                exchange.rate,
            )?;
            txn.transferred_amount(-spent);
        } else if let Some(spent) = entry.spent {
            txn.transferred_amount(-spent);
        }
        if let Some(fee) = entry.fee {
            let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
                "config should have operator to have charge",
            ))?;
            txn.add_charge(payee, fee.amount);
        }
        result.push(txn);
    }
    Ok(result)
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
        let pattern = extract::regex_matcher(v)?;
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
            Field::Payee => fragment.payee.unwrap_or(entity.payee.as_str()),
            Field::Category => entity.category.as_str(),
        };
        self.pattern.captures(target).map(Into::into)
    }
}
