pub struct CSVImporter {}

use super::config;
use super::ImportError;
use crate::data;
use data::parse_comma_decimal;

use chrono::NaiveDate;
use rust_decimal::Decimal;

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
    config_mapping: &HashMap<config::FieldKey, String>,
    header: &csv::StringRecord,
) -> Result<FieldMap, ImportError> {
    let hm: HashMap<&str, usize> = header.iter().enumerate().map(|(k, v)| (v, k)).collect();
    let ki: HashMap<config::FieldKey, usize> = config_mapping
        .iter()
        .filter_map(|(&k, name)| hm.get(name.as_str()).map(|&i| (k, i)))
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

impl super::Importer for CSVImporter {
    fn import<R: std::io::Read>(
        &self,
        r: &mut R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<data::Transaction>, ImportError> {
        let mut res = Vec::new();
        let mut rdr = csv::ReaderBuilder::new().delimiter(b',').from_reader(r);
        if !rdr.has_headers() {
            return Err(ImportError::Other("no header of CSV".to_string()));
        }
        let header = rdr.headers()?;
        let fm = resolve_fields(&config.field, header)?;
        let size = fm.max();
        for may_record in rdr.records() {
            let r = may_record?;
            if r.len() <= size {
                return Err(ImportError::Other("unexpected csv length".to_string()));
            }
            let date = NaiveDate::parse_from_str(r.get(fm.date).unwrap(), "%Y年%m月%d日")?;
            let payee = r.get(fm.payee).unwrap();
            let has_credit = r.get(fm.credit).unwrap() != "";
            let has_debit = r.get(fm.debit).unwrap() != "";
            let balance = parse_comma_decimal(r.get(fm.balance).unwrap())?;
            let mut posts = Vec::new();
            if has_credit {
                let credit: Decimal = parse_comma_decimal(r.get(fm.credit).unwrap())?;
                posts.push(data::Post {
                    account: "Incomes:Unknown".to_string(),
                    amount: data::Amount {
                        value: -credit,
                        commodity: config.commodity.clone(),
                    },
                    balance: None,
                });
                posts.push(data::Post {
                    account: "Assets:Banks:MyBank".to_string(),
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
                posts.push(data::Post {
                    account: "Assets:Banks:MyBank".to_string(),
                    amount: data::Amount {
                        value: -debit,
                        commodity: config.commodity.clone(),
                    },
                    balance: Some(data::Amount {
                        value: balance,
                        commodity: config.commodity.clone(),
                    }),
                });
                posts.push(data::Post {
                    account: "Expenses:Unknown".to_string(),
                    amount: data::Amount {
                        value: debit,
                        commodity: config.commodity.clone(),
                    },
                    balance: None,
                });
            } else {
                // warning log or error?
                return Err(ImportError::Other("credit and debit both zero".to_string()));
            }
            res.push(data::Transaction {
                date: date,
                payee: payee.to_string(),
                posts: posts,
            });
        }
        return Ok(res);
    }
}
