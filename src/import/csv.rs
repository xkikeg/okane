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
            let has_credit = r.get(fm.credit).unwrap() != "";
            let has_debit = r.get(fm.debit).unwrap() != "";
            let balance = parse_comma_decimal(r.get(fm.balance).unwrap())?;
            let fragment = rewriter.rewrite(original_payee);
            if fragment.account.is_none() {
                warn!("account unmatched at line {}, payee={}", pos.line(), original_payee);
            }
            if has_credit {
                let credit: Decimal = parse_comma_decimal(r.get(fm.credit).unwrap())?;
                res.push(single_entry::Txn {
                    date: date,
                    code: fragment.code,
                    payee: fragment.payee,
                    dest_account: fragment.account,
                    amount: data::Amount {
                        value: credit,
                        commodity: config.commodity.clone(),
                    },
                    balance: Some(data::Amount {
                        value: balance,
                        commodity: config.commodity.clone(),
                    }),
                });
            } else if has_debit {
                let debit: Decimal = parse_comma_decimal(r.get(fm.debit).unwrap())?;
                res.push(single_entry::Txn {
                    date: date,
                    code: fragment.code,
                    payee: fragment.payee,
                    dest_account: fragment.account,
                    amount: data::Amount {
                        value: -debit,
                        commodity: config.commodity.clone(),
                    },
                    balance: Some(data::Amount {
                        value: balance,
                        commodity: config.commodity.clone(),
                    }),
                });
            } else {
                // warning log or error?
                return Err(ImportError::Other("credit and debit both zero".to_string()));
            }
        }
        return Ok(res);
    }
}

#[derive(PartialEq, Debug)]
struct FieldMap {
    date: usize,
    payee: usize,
    credit: usize,
    debit: usize,
    balance: usize,
}

impl FieldMap {
    fn max(&self) -> usize {
        return *[self.date, self.payee, self.credit, self.debit, self.balance]
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
    let credit = ki
        .get(&config::FieldKey::Credit)
        .ok_or(ImportError::InvalidConfig("no Credit field specified"))?;
    let debit = ki
        .get(&config::FieldKey::Debit)
        .ok_or(ImportError::InvalidConfig("no Debit field specified"))?;
    let balance = ki
        .get(&config::FieldKey::Balance)
        .ok_or(ImportError::InvalidConfig("no Balance field specified"))?;
    return Ok(FieldMap {
        date: *date,
        payee: *payee,
        credit: *credit,
        debit: *debit,
        balance: *balance,
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
                credit: 2,
                debit: 3,
                balance: 4,
            }
        );
    }

    #[test]
    #[ignore]
    fn test_resolve_fields_index_amount() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Index(0),
            FieldKey::Payee => FieldPos::Index(1),
            FieldKey::Amount => FieldPos::Index(2),
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
                credit: 2,
                debit: 3,
                balance: 4,
            }
        );
    }
}
