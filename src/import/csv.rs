use super::config;
use super::extract;
use super::single_entry;
use super::ImportError;
use crate::data;
use data::parse_comma_decimal;

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use chrono::NaiveDate;
use log::{info, warn};
use regex::Regex;
use rust_decimal::Decimal;

pub struct CsvImporter {}

impl super::Importer for CsvImporter {
    fn import<R: std::io::Read>(
        &self,
        r: &mut R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError> {
        let mut res: Vec<single_entry::Txn> = Vec::new();
        let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(r);
        if !rdr.has_headers() {
            return Err(ImportError::Other("no header of CSV".to_string()));
        }
        let header = rdr.headers()?;
        let fm = resolve_fields(&config.format.fields, header)?;
        let size = fm.max();
        let extractor: extract::Extractor<CsvMatcher> = (&config.rewrite).try_into()?;
        for may_record in rdr.records() {
            let r = may_record?;
            let pos = r.position().expect("csv record position");
            if r.len() <= size {
                return Err(ImportError::Other(format!(
                    "csv record length too short at line {}",
                    pos.line()
                )));
            }
            let datestr = r.get(fm.date).unwrap();
            if datestr.is_empty() {
                info!("skip empty date at line {}", pos.line());
                continue;
            }
            let date = NaiveDate::parse_from_str(datestr, config.format.date.as_str())?;
            let original_payee = r.get(fm.payee).unwrap();
            let amount = fm.value.amount(config.account_type, &r)?;
            let balance = fm
                .balance
                .map(|b| parse_comma_decimal(r.get(b).unwrap()))
                .transpose()?;
            let fragment = extractor.extract(Payee(original_payee));
            if fragment.account.is_none() {
                warn!(
                    "account unmatched at line {}, payee={}",
                    pos.line(),
                    original_payee
                );
            }
            let mut txn = single_entry::Txn::new(
                date,
                fragment.payee.unwrap_or(original_payee),
                data::Amount {
                    value: amount,
                    commodity: config.commodity.clone(),
                },
            );
            txn.code_option(fragment.code)
                .dest_account_option(fragment.account);
            if !fragment.cleared {
                txn.clear_state(data::ClearState::Pending);
            }
            if let Some(b) = balance {
                txn.balance(data::Amount {
                    value: b,
                    commodity: config.commodity.clone(),
                });
            }
            res.push(txn);
        }
        Ok(res)
    }
}

#[derive(PartialEq, Debug)]
enum FieldMapValues {
    CreditDebit { credit: usize, debit: usize },
    Amount(usize),
}

impl FieldMapValues {
    fn max(&self) -> usize {
        match self {
            FieldMapValues::CreditDebit { credit, debit } => std::cmp::max(*credit, *debit),
            FieldMapValues::Amount(amount) => *amount,
        }
    }

    fn amount(
        &self,
        at: config::AccountType,
        r: &csv::StringRecord,
    ) -> Result<Decimal, ImportError> {
        match self {
            FieldMapValues::CreditDebit { credit, debit } => {
                let credit = r.get(*credit).unwrap();
                let debit = r.get(*debit).unwrap();
                if !credit.is_empty() {
                    Ok(parse_comma_decimal(credit)?)
                } else if !debit.is_empty() {
                    Ok(-parse_comma_decimal(debit)?)
                } else {
                    Err(ImportError::Other("credit and debit both zero".to_string()))
                }
            }
            FieldMapValues::Amount(a) => {
                let amount = parse_comma_decimal(r.get(*a).unwrap())?;
                Ok(match at {
                    config::AccountType::Asset => amount,
                    config::AccountType::Liability => -amount,
                })
            }
        }
    }
}

#[derive(PartialEq, Debug)]
struct FieldMap {
    date: usize,
    payee: usize,
    value: FieldMapValues,
    balance: Option<usize>,
}

impl FieldMap {
    fn max(&self) -> usize {
        return *[
            self.date,
            self.payee,
            self.value.max(),
            self.balance.unwrap_or(0),
        ]
        .iter()
        .max()
        .unwrap();
    }
}

fn resolve_fields(
    config_mapping: &HashMap<config::FieldKey, config::FieldPos>,
    header: &csv::StringRecord,
) -> Result<FieldMap, ImportError> {
    let hm: HashMap<&str, usize> = header.iter().enumerate().map(|(k, v)| (v, k)).collect();
    let mut actual_labels: Vec<&str> = hm.keys().cloned().collect();
    actual_labels.sort_unstable();
    let mut not_found_labels: Vec<&str> = config_mapping
        .iter()
        .filter_map(|kv| match &kv.1 {
            config::FieldPos::Label(label) if !hm.contains_key(label.as_str()) => {
                Some(label.as_str())
            }
            _ => None,
        })
        .collect();
    not_found_labels.sort_unstable();
    if !not_found_labels.is_empty() {
        return Err(ImportError::Other(format!(
            "specified labels not found: {} actual labels: {}",
            not_found_labels.join(","),
            actual_labels.join(",")
        )));
    }
    let ki: HashMap<config::FieldKey, usize> = config_mapping
        .iter()
        .filter_map(|(&k, pos)| {
            match &pos {
                config::FieldPos::Index(i) => Some(*i),
                config::FieldPos::Label(label) => hm.get(label.as_str()).cloned(),
            }
            .map(|i| (k, i))
        })
        .collect();
    let date = ki
        .get(&config::FieldKey::Date)
        .ok_or(ImportError::InvalidConfig("no Date field specified"))?;
    let payee = ki
        .get(&config::FieldKey::Payee)
        .ok_or(ImportError::InvalidConfig("no Payee field specified"))?;
    let amount = ki.get(&config::FieldKey::Amount);
    let credit = ki.get(&config::FieldKey::Credit);
    let debit = ki.get(&config::FieldKey::Debit);
    let value = match amount {
        Some(&a) => Ok(FieldMapValues::Amount(a)),
        None => credit
            .zip(debit)
            .map(|(c, d)| FieldMapValues::CreditDebit {
                credit: *c,
                debit: *d,
            })
            .ok_or(ImportError::InvalidConfig(
                "either amount or credit/debit pair should be set",
            )),
    }?;
    let balance = ki.get(&config::FieldKey::Balance);
    Ok(FieldMap {
        date: *date,
        payee: *payee,
        value,
        balance: balance.cloned(),
    })
}

/// Payee is a wrapper to represent payee.
/// In future we'll use CSV-row instead of just a payee.
#[derive(Debug, Copy, Clone)]
struct Payee<'a>(&'a str);

/// Matcher for CSV, currently it only checks payee.
#[derive(Debug)]
struct CsvMatcher(regex::Regex);

impl<'a> extract::Entity<'a> for CsvMatcher {
    type T = Payee<'a>;
}

impl TryFrom<(config::RewriteField, &str)> for CsvMatcher {
    type Error = ImportError;

    fn try_from(from: (config::RewriteField, &str)) -> Result<CsvMatcher, ImportError> {
        if from.0 == config::RewriteField::Payee {
            let re = Regex::new(from.1)?;
            Ok(CsvMatcher(re))
        } else {
            Err(ImportError::Other(format!(
                "unsupported matcher field: {}",
                from.0
            )))
        }
    }
}

impl extract::EntityMatcher for CsvMatcher {
    fn captures<'a>(
        &self,
        fragment: &extract::Fragment<'a>,
        entity: Payee<'a>,
    ) -> Option<extract::Matched<'a>> {
        let payee = fragment.payee.unwrap_or(entity.0);
        self.0.captures(payee).map(|c| c.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::*;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_resolve_fields_label_credit_debit() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Label("日付".to_owned()),
            FieldKey::Payee => FieldPos::Label("摘要".to_owned()),
            FieldKey::Credit => FieldPos::Label("お預け入れ額".to_owned()),
            FieldKey::Debit => FieldPos::Label("お引き出し額".to_owned()),
            FieldKey::Balance => FieldPos::Label("残高".to_owned()),
        };
        let got = resolve_fields(
            &config,
            &csv::StringRecord::from(vec!["日付", "摘要", "お預け入れ額", "お引き出し額", "残高"]),
        )
        .unwrap();
        assert_eq!(
            got,
            FieldMap {
                date: 0,
                payee: 1,
                value: FieldMapValues::CreditDebit {
                    credit: 2,
                    debit: 3
                },
                balance: Some(4),
            }
        );
    }

    #[test]
    fn test_resolve_fields_index_amount() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Index(0),
            FieldKey::Payee => FieldPos::Index(1),
            FieldKey::Amount => FieldPos::Index(2),
        };
        let got = resolve_fields(&config, &csv::StringRecord::from(vec!["unrelated"])).unwrap();
        assert_eq!(
            got,
            FieldMap {
                date: 0,
                payee: 1,
                value: FieldMapValues::Amount(2),
                balance: None,
            }
        );
    }
}
