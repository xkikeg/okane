use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::BufRead;
use std::io::BufReader;

use chrono::NaiveDate;
use log::{info, warn};
use regex::Regex;
use rust_decimal::Decimal;

use okane_core::datamodel;
use okane_core::repl;
use repl::pretty_decimal::{self, PrettyDecimal};

use super::config;
use super::extract;
use super::single_entry::{self, CommodityPair};
use super::ImportError;

pub struct CsvImporter {}

fn str_to_comma_decimal(input: &str) -> Result<Decimal, pretty_decimal::Error> {
    let r: Result<PrettyDecimal, pretty_decimal::Error> = input.parse();
    r.map(|x| x.into())
}

impl super::Importer for CsvImporter {
    fn import<R: std::io::Read>(
        &self,
        r: R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError> {
        let mut res: Vec<single_entry::Txn> = Vec::new();
        let mut br = BufReader::new(r);
        let mut rb = csv::ReaderBuilder::new();
        rb.flexible(true);
        if !config.format.delimiter.is_empty() {
            rb.delimiter(config.format.delimiter.as_bytes()[0]);
        }
        if config.format.skip.head > 0 {
            let mut skipped = String::new();
            for i in 0..config.format.skip.head {
                skipped.clear();
                br.read_line(&mut skipped)?;
                log::info!("skipped {}-th line: {}", i, skipped.as_str().trim_end());
            }
        }
        let mut rdr = rb.from_reader(br);
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
                    "csv record length too short at line {}: want {}, got {}",
                    pos.line(),
                    size,
                    r.len()
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
                .map(|i| str_to_comma_decimal(r.get(i).unwrap()))
                .transpose()?;
            let secondary_amount = fm
                .secondary_amount
                .map(|i| str_to_comma_decimal(r.get(i).unwrap()))
                .transpose()?;
            let secondary_commodity = fm.secondary_commodity.and_then(|i| r.get(i));
            let category = fm.category.and_then(|i| r.get(i));
            let commodity = fm
                .commodity
                .and_then(|i| r.get(i))
                .map(|x| x.to_string())
                .unwrap_or_else(|| config.commodity.primary.clone());
            let rate = fm
                .rate
                .map(|i| str_to_comma_decimal(r.get(i).unwrap()))
                .transpose()?;
            let fragment = extractor.extract(Record {
                payee: original_payee,
                category,
            });
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
                datamodel::Amount {
                    value: amount,
                    commodity: commodity.clone(),
                },
            );
            txn.code_option(fragment.code)
                .dest_account_option(fragment.account);
            if !fragment.cleared {
                txn.clear_state(datamodel::ClearState::Pending);
            }
            if let Some(b) = balance {
                txn.balance(datamodel::Amount {
                    value: b,
                    commodity: commodity.clone(),
                });
            }
            if let Some(conv) = fragment.conversion {
                let rate = rate.ok_or_else(|| {
                    ImportError::Other(format!(
                        "no rate specified for transcation with conversion: line {}",
                        pos.line()
                    ))
                })?;
                let secondary_commodity = conv.commodity.as_deref().or(secondary_commodity)
                    .ok_or_else(||ImportError::Other(format!("either rewrite.conversion.commodity or secondary_commodity field must be set @ line {}", pos.line())))?;
                let (rate_key, computed_transferred) = match conv.rate {
                    config::ConversionRateSpec::PriceOfPrimary => (
                        CommodityPair {
                            source: secondary_commodity.to_owned(),
                            target: commodity.to_owned(),
                        },
                        amount * rate,
                    ),
                    config::ConversionRateSpec::PriceOfSecondary => (
                        CommodityPair {
                            source: commodity.to_owned(),
                            target: secondary_commodity.to_owned(),
                        },
                        amount / rate,
                    ),
                };
                txn.add_rate(rate_key, rate)?;
                let transferred = match conv.amount {
                    config::ConversionAmountSpec::Extract => secondary_amount.ok_or_else(|| ImportError::Other(format!(
                            "secondary_amount should be specified when conversion.amount is set to extract @ line {}", pos.line()
                    )))?,
                    config::ConversionAmountSpec::Compute => computed_transferred,
                };
                txn.transferred_amount(datamodel::Amount {
                    value: transferred,
                    commodity: secondary_commodity.to_owned(),
                });
            }
            res.push(txn);
        }
        match config.format.row_order {
            config::RowOrder::OldToNew => (),
            config::RowOrder::NewToOld => {
                res.reverse();
            }
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
                    Ok(str_to_comma_decimal(credit)?)
                } else if !debit.is_empty() {
                    Ok(-str_to_comma_decimal(debit)?)
                } else {
                    Err(ImportError::Other("credit and debit both zero".to_string()))
                }
            }
            FieldMapValues::Amount(a) => {
                let amount = str_to_comma_decimal(r.get(*a).unwrap())?;
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
    category: Option<usize>,
    commodity: Option<usize>,
    rate: Option<usize>,
    secondary_amount: Option<usize>,
    secondary_commodity: Option<usize>,
}

impl FieldMap {
    fn max(&self) -> usize {
        return *[
            self.date,
            self.payee,
            self.value.max(),
            self.balance.unwrap_or(0),
            self.commodity.unwrap_or(0),
            self.category.unwrap_or(0),
            self.rate.unwrap_or(0),
            self.secondary_amount.unwrap_or(0),
            self.secondary_commodity.unwrap_or(0),
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
    let category = ki.get(&config::FieldKey::Category).cloned();
    let balance = ki.get(&config::FieldKey::Balance).cloned();
    let commodity = ki.get(&config::FieldKey::Commodity).cloned();
    let rate = ki.get(&config::FieldKey::Rate).cloned();
    let secondary_amount = ki.get(&config::FieldKey::SecondaryAmount).cloned();
    let secondary_commodity = ki.get(&config::FieldKey::SecondaryCommodity).cloned();
    Ok(FieldMap {
        date: *date,
        payee: *payee,
        value,
        balance,
        category,
        commodity,
        rate,
        secondary_amount,
        secondary_commodity,
    })
}

/// Record is a wrapper to represent a CSV line.
#[derive(Debug, Copy, Clone)]
struct Record<'a> {
    payee: &'a str,
    category: Option<&'a str>,
}

#[derive(Debug)]
enum Field {
    Payee,
    Category,
}

/// Matcher for CSV.
#[derive(Debug)]
struct CsvMatcher {
    field: Field,
    pattern: Regex,
}

impl<'a> extract::Entity<'a> for CsvMatcher {
    type T = Record<'a>;
}

impl TryFrom<(config::RewriteField, &str)> for CsvMatcher {
    type Error = ImportError;

    fn try_from((field, pattern): (config::RewriteField, &str)) -> Result<CsvMatcher, Self::Error> {
        let field = match field {
            config::RewriteField::Payee => Ok(Field::Payee),
            config::RewriteField::Category => Ok(Field::Category),
            _ => Err(ImportError::InvalidConfig(
                "CSV only supports payee or category matcher.",
            )),
        }?;
        let pattern = extract::regex_matcher(pattern)?;
        Ok(CsvMatcher { field, pattern })
    }
}

impl extract::EntityMatcher for CsvMatcher {
    fn captures<'a>(
        &self,
        fragment: &extract::Fragment<'a>,
        entity: Record<'a>,
    ) -> Option<extract::Matched<'a>> {
        match self.field {
            Field::Payee => {
                let payee = fragment.payee.unwrap_or(entity.payee);
                self.pattern.captures(payee).map(|c| c.into())
            }
            Field::Category => entity
                .category
                .and_then(|c| self.pattern.captures(c))
                .and(Some(extract::Matched::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::*;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[test]
    fn resolve_fields_label_credit_debit() {
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
                category: None,
                commodity: None,
                rate: None,
                secondary_amount: None,
                secondary_commodity: None,
            }
        );
    }

    #[test]
    fn resolve_fields_index_amount() {
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
                category: None,
                commodity: None,
                rate: None,
                secondary_amount: None,
                secondary_commodity: None,
            }
        );
    }

    #[test]
    fn resolve_fields_optionals() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Index(0),
            FieldKey::Payee => FieldPos::Index(1),
            FieldKey::Amount => FieldPos::Index(2),
            FieldKey::Balance => FieldPos::Index(3),
            FieldKey::Category => FieldPos::Index(4),
            FieldKey::Commodity => FieldPos::Index(5),
            FieldKey::Rate => FieldPos::Index(6),
            FieldKey::SecondaryAmount => FieldPos::Index(7),
            FieldKey::SecondaryCommodity => FieldPos::Index(8),
        };
        let got = resolve_fields(&config, &csv::StringRecord::from(vec!["unrelated"])).unwrap();
        assert_eq!(
            got,
            FieldMap {
                date: 0,
                payee: 1,
                value: FieldMapValues::Amount(2),
                balance: Some(3),
                category: Some(4),
                commodity: Some(5),
                rate: Some(6),
                secondary_amount: Some(7),
                secondary_commodity: Some(8),
            }
        );
    }
}
