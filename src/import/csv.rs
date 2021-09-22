use super::config;
use super::single_entry;
use super::ImportError;
use crate::data;
use data::parse_comma_decimal;

use chrono::NaiveDate;
use log::{info, warn};
use rust_decimal::Decimal;

pub struct CSVImporter {}

impl super::Importer for CSVImporter {
    fn import<R: std::io::Read>(
        &self,
        r: &mut R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError> {
        let mut res = Vec::new();
        let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(r);
        if !rdr.has_headers() {
            return Err(ImportError::Other("no header of CSV".to_string()));
        }
        let header = rdr.headers()?;
        let fm = resolve_fields(&config.format.fields, header)?;
        let size = fm.max();
        let rewriter = new_rewriter(config)?;
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
            let amount = fm.value.get_amount(config.account_type, &r)?;
            let balance = fm
                .balance
                .map(|b| parse_comma_decimal(r.get(b).unwrap()))
                .transpose()?;
            let fragment = rewriter.rewrite(original_payee);
            if fragment.account.is_none() {
                warn!(
                    "account unmatched at line {}, payee={}",
                    pos.line(),
                    original_payee
                );
            }
            res.push(single_entry::Txn {
                date: date,
                code: fragment.code,
                payee: fragment.payee,
                dest_account: fragment.account,
                amount: data::Amount {
                    value: amount,
                    commodity: config.commodity.clone(),
                },
                balance: balance.map(|v| data::Amount {
                    value: v,
                    commodity: config.commodity.clone(),
                }),
            });
        }
        return Ok(res);
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

    fn get_amount(&self, at: config::AccountType, r: &csv::StringRecord) -> Result<Decimal, ImportError> {
        match self {
            FieldMapValues::CreditDebit { credit, debit } => {
                let credit = r.get(*credit).unwrap();
                let debit = r.get(*debit).unwrap();
                if !credit.is_empty() {
                    return Ok(parse_comma_decimal(credit)?);
                } else if !debit.is_empty() {
                    return Ok(-parse_comma_decimal(debit)?);
                } else {
                    return Err(ImportError::Other("credit and debit both zero".to_string()));
                }
            }
            FieldMapValues::Amount(a) => {
                let amount = parse_comma_decimal(r.get(*a).unwrap())?;
                Ok(match at {
                    config::AccountType::Asset => amount,
                    config::AccountType::Liability => -amount,
                })
            },
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

use std::collections::HashMap;

fn resolve_fields(
    config_mapping: &HashMap<config::FieldKey, config::FieldPos>,
    header: &csv::StringRecord,
) -> Result<FieldMap, ImportError> {
    let hm: HashMap<&str, usize> = header.iter().enumerate().map(|(k, v)| (v, k)).collect();
    let ki: HashMap<config::FieldKey, usize> = config_mapping
        .iter()
        .filter_map(|(&k, pos)| {
            match &pos {
                config::FieldPos::Index(i) => Some(*i as usize),
                config::FieldPos::Label(label) => hm.get(label.as_str()).map(Clone::clone),
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
    return Ok(FieldMap {
        date: *date,
        payee: *payee,
        value: value,
        balance: balance.cloned(),
    });
}

struct RewriteElement {
    payee: regex::Regex,
    account: Option<String>,
}
/// Holds rewrite config to provide rewrite method.
struct Rewriter {
    elems: Vec<RewriteElement>,
}

fn new_rewriter(config: &config::ConfigEntry) -> Result<Rewriter, ImportError> {
    let mut elems = Vec::new();
    for rw in &config.rewrite {
        let re = regex::Regex::new(rw.payee.as_str())
            .or(Err(ImportError::InvalidConfig("cannot compile regex")))?;
        elems.push(RewriteElement {
            payee: re,
            account: rw.account.clone(),
        });
    }
    return Ok(Rewriter { elems: elems });
}

#[derive(Debug)]
struct Fragment {
    payee: String,
    code: Option<String>,
    account: Option<String>,
}

impl Rewriter {
    fn rewrite(&self, payee: &str) -> Fragment {
        let mut payee = payee;
        let mut code = None;
        let mut account = None;
        for elem in &self.elems {
            if let Some(c) = elem.payee.captures(payee) {
                if let Some(p) = c.name("payee") {
                    payee = p.as_str();
                }
                if let Some(p) = c.name("code") {
                    code = Some(p.as_str().to_string());
                }
                if let Some(a) = &elem.account {
                    account = Some(a.clone());
                }
            }
        }
        return Fragment {
            payee: payee.to_string(),
            code: code,
            account: account,
        };
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
