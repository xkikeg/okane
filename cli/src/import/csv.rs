use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::BufRead;
use std::io::BufReader;

use chrono::NaiveDate;
use log::{info, warn};
use regex::Regex;
use rust_decimal::Decimal;

use okane_core::syntax;

use super::amount::OwnedAmount;
use super::config::{self, FieldKey, TemplateField};
use super::single_entry::{self, CommodityPair};
use super::ImportError;
use super::{
    extract,
    template::{self, Template, TemplateKey},
};

fn str_to_comma_decimal(input: &str) -> Result<Option<Decimal>, ImportError> {
    if input.is_empty() {
        return Ok(None);
    }
    let a: syntax::expr::Amount = input
        .try_into()
        .map_err(|e| ImportError::Other(format!("failed to parse comma decimal: {}", e)))?;
    Ok(Some(a.value.value))
}

pub fn import<R: std::io::Read>(
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
    let fm = FieldMap::try_new(&config.format.fields, header)?;
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
        let datestr = fm
            .extract(FieldKey::Date, &r)?
            .ok_or_else(|| ImportError::Other("Field date must be present".to_string()))?;
        if datestr.is_empty() {
            info!("skip empty date at line {}", pos.line());
            continue;
        }
        let date = NaiveDate::parse_from_str(&datestr, config.format.date.as_str())?;
        let original_payee = fm
            .extract(FieldKey::Payee, &r)?
            .ok_or_else(|| ImportError::Other("Field payee must be present".to_string()))?;
        let amount = fm.amount(config.account_type, &r)?;
        let balance = fm
            .extract(FieldKey::Balance, &r)?
            .map_or(Ok(None), |s| str_to_comma_decimal(&s))?;
        let secondary_amount = fm
            .extract(FieldKey::SecondaryAmount, &r)?
            .map_or(Ok(None), |s| str_to_comma_decimal(&s))?;
        let secondary_commodity = fm.extract(FieldKey::SecondaryCommodity, &r)?;
        let category = fm.extract(FieldKey::Category, &r)?;
        let commodity = fm
            .extract(FieldKey::Commodity, &r)?
            .unwrap_or_else(|| Cow::Borrowed(&config.commodity.primary));
        let rate = fm
            .extract(FieldKey::Rate, &r)?
            .map_or_else(|| Ok(None), |s| str_to_comma_decimal(&s))?;
        let fragment = extractor.extract(Record {
            payee: &original_payee,
            category: category.as_deref(),
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
            fragment.payee.unwrap_or(&original_payee),
            OwnedAmount {
                value: amount,
                commodity: commodity.clone().into_owned(),
            },
        );
        txn.code_option(fragment.code)
            .dest_account_option(fragment.account);
        if !fragment.cleared {
            txn.clear_state(syntax::ClearState::Pending);
        }
        if let Some(b) = balance {
            txn.balance(OwnedAmount {
                value: b,
                commodity: commodity.clone().into_owned(),
            });
        }
        if let Some(charge) = fm.extract(FieldKey::Charge, &r)? {
            let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
                "config should have operator to have charge",
            ))?;
            if let Some(value) = str_to_comma_decimal(&charge)? {
                txn.add_charge(
                    payee,
                    OwnedAmount {
                        value,
                        commodity: commodity.clone().into_owned(),
                    },
                );
            }
        }
        if let Some(conv) = fragment.conversion {
            let rate = rate.ok_or_else(|| {
                ImportError::Other(format!(
                    "no rate specified for transcation with conversion: line {}",
                    pos.line()
                ))
            })?;
            let secondary_commodity = conv.commodity.as_deref().or(secondary_commodity.as_deref())
                    .ok_or_else(||ImportError::Other(format!("either rewrite.conversion.commodity or secondary_commodity field must be set @ line {}", pos.line())))?;
            let (rate_key, computed_transferred) = match conv.rate {
                config::ConversionRateSpec::PriceOfPrimary => (
                    CommodityPair {
                        source: secondary_commodity.to_owned(),
                        target: commodity.into_owned(),
                    },
                    amount * rate,
                ),
                config::ConversionRateSpec::PriceOfSecondary => (
                    CommodityPair {
                        source: commodity.into_owned(),
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
            txn.transferred_amount(OwnedAmount {
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

#[derive(Debug, PartialEq)]
enum TxnValueField {
    CreditDebit { credit: Field, debit: Field },
    Amount(Field),
}

#[derive(Debug, PartialEq, Clone)]
enum Field {
    ColumnIndex(usize),
    Template(Template),
}

struct MappedRecord<'a> {
    field_map: &'a FieldMap,
    record: &'a csv::StringRecord,
}

impl<'a> template::RenderValue<'a> for MappedRecord<'a> {
    fn query(&self, key: TemplateKey) -> Option<&'a str> {
        let key = match key {
            // Clone here might have penalty on template,
            // but it will be a failure path so good to go.
            TemplateKey::Named(fk) => self.field_map.field(fk)?.clone(),
            TemplateKey::Indexed(i) => Field::ColumnIndex(i.as_zero_based()),
        };
        match key {
            // It might cause inifite loop if we resolve another template during template rendering.
            // Thus we only support simple column reference.
            Field::Template(_) => None,
            Field::ColumnIndex(c) => self.record.get(c),
        }
    }
}

#[derive(Debug, PartialEq)]
struct FieldMap {
    date: Field,
    payee: Field,
    value: TxnValueField,
    all_fields: HashMap<FieldKey, Field>,
    max_column: usize,
}

impl FieldMap {
    fn max(&self) -> usize {
        self.max_column
    }

    fn field(&self, field_key: FieldKey) -> Option<&Field> {
        self.all_fields.get(&field_key)
    }

    fn extract<'a>(
        &self,
        field_key: FieldKey,
        record: &'a csv::StringRecord,
    ) -> Result<Option<Cow<'a, str>>, template::RenderError> {
        self.field(field_key)
            .and_then(|x| self.resolve(field_key, x, record).transpose())
            .transpose()
    }

    fn resolve<'a>(
        &self,
        field_key: FieldKey,
        field: &Field,
        record: &'a csv::StringRecord,
    ) -> Result<Option<Cow<'a, str>>, template::RenderError> {
        match field {
            Field::ColumnIndex(i) => Ok(record.get(*i).map(Cow::Borrowed)),
            Field::Template(template) => template
                .render(
                    field_key,
                    MappedRecord {
                        field_map: self,
                        record,
                    },
                )
                .map(|x| Some(Cow::Owned(format!("{}", x)))),
        }
    }

    fn amount(
        &self,
        at: config::AccountType,
        r: &csv::StringRecord,
    ) -> Result<Decimal, ImportError> {
        match &self.value {
            TxnValueField::CreditDebit { credit, debit } => {
                let credit = self
                    .resolve(FieldKey::Credit, credit, r)?
                    .ok_or_else(|| ImportError::Other("Field credit must exist".to_string()))?;
                let debit = self
                    .resolve(FieldKey::Debit, debit, r)?
                    .ok_or_else(|| ImportError::Other("Field debit must exist".to_string()))?;
                if !credit.is_empty() {
                    Ok(str_to_comma_decimal(&credit)?.unwrap_or(Decimal::ZERO))
                } else if !debit.is_empty() {
                    Ok(-str_to_comma_decimal(&debit)?.unwrap_or(Decimal::ZERO))
                } else {
                    Err(ImportError::Other(
                        "either credit or debit must be non-empty".to_string(),
                    ))
                }
            }
            TxnValueField::Amount(a) => {
                let s = self
                    .resolve(FieldKey::Amount, a, r)?
                    .ok_or_else(|| ImportError::Other("Field amount must exist".to_string()))?;
                let amount = str_to_comma_decimal(&s)?.unwrap_or(Decimal::ZERO);
                Ok(match at {
                    config::AccountType::Asset => amount,
                    config::AccountType::Liability => -amount,
                })
            }
        }
    }

    fn try_new(
        config_mapping: &HashMap<config::FieldKey, config::FieldPos>,
        header: &csv::StringRecord,
    ) -> Result<Self, ImportError> {
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
        let mut ki: HashMap<config::FieldKey, Field> = HashMap::with_capacity(config_mapping.len());
        for (&k, pos) in config_mapping {
            let field = match &pos {
                config::FieldPos::Index(i) => Ok(Field::ColumnIndex(i.as_zero_based())),
                config::FieldPos::Label(label) => hm
                    .get(label.as_str())
                    .cloned()
                    .map(Field::ColumnIndex)
                    .ok_or_else(|| {
                        ImportError::Other(format!("failed to find the field {}", label))
                    }),
                config::FieldPos::Template(TemplateField { template }) => {
                    let template = template.parse()?;
                    Ok(Field::Template(template))
                }
            }?;
            ki.insert(k, field);
        }
        let max_column = ki
            .values()
            .filter_map(|x| match x {
                Field::ColumnIndex(x) => Some(x),
                Field::Template(_) => None,
            })
            .copied()
            .max()
            .unwrap_or(0);
        let date = ki
            .get(&config::FieldKey::Date)
            .cloned()
            .ok_or(ImportError::InvalidConfig("no Date field specified"))?;
        let payee = ki
            .get(&config::FieldKey::Payee)
            .cloned()
            .ok_or(ImportError::InvalidConfig("no Payee field specified"))?;
        let amount = ki.get(&config::FieldKey::Amount).cloned();
        let credit = ki.get(&config::FieldKey::Credit).cloned();
        let debit = ki.get(&config::FieldKey::Debit).cloned();
        let value = match amount {
            Some(a) => Ok(TxnValueField::Amount(a)),
            None => credit
                .zip(debit)
                .map(|(c, d)| TxnValueField::CreditDebit {
                    credit: c,
                    debit: d,
                })
                .ok_or(ImportError::InvalidConfig(
                    "either amount or credit/debit pair should be set",
                )),
        }?;
        Ok(FieldMap {
            date,
            payee,
            value,
            all_fields: ki,
            max_column,
        })
    }
}

/// Record is a wrapper to represent a CSV line.
#[derive(Debug, Copy, Clone)]
struct Record<'a> {
    payee: &'a str,
    category: Option<&'a str>,
}

#[derive(Debug)]
enum MatchField {
    Payee,
    Category,
}

/// Matcher for CSV.
#[derive(Debug)]
struct CsvMatcher {
    field: MatchField,
    pattern: Regex,
}

impl<'a> extract::Entity<'a> for CsvMatcher {
    type T = Record<'a>;
}

impl TryFrom<(config::RewriteField, &str)> for CsvMatcher {
    type Error = ImportError;

    fn try_from((field, pattern): (config::RewriteField, &str)) -> Result<CsvMatcher, Self::Error> {
        let field = match field {
            config::RewriteField::Payee => Ok(MatchField::Payee),
            config::RewriteField::Category => Ok(MatchField::Category),
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
            MatchField::Payee => {
                let payee = fragment.payee.unwrap_or(entity.payee);
                self.pattern.captures(payee).map(|c| c.into())
            }
            MatchField::Category => entity
                .category
                .and_then(|c| self.pattern.captures(c))
                .and(Some(extract::Matched::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::config::{AccountType, FieldPos};
    use crate::one_based;

    #[test]
    fn field_map_try_new_label_credit_debit() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Label("日付".to_owned()),
            FieldKey::Payee => FieldPos::Label("摘要".to_owned()),
            FieldKey::Credit => FieldPos::Label("お預け入れ額".to_owned()),
            FieldKey::Debit => FieldPos::Label("お引き出し額".to_owned()),
            FieldKey::Balance => FieldPos::Label("残高".to_owned()),
        };
        let got = FieldMap::try_new(
            &config,
            &csv::StringRecord::from(vec!["日付", "摘要", "お預け入れ額", "お引き出し額", "残高"]),
        )
        .unwrap();
        assert_eq!(
            FieldMap {
                date: Field::ColumnIndex(0),
                payee: Field::ColumnIndex(1),
                value: TxnValueField::CreditDebit {
                    credit: Field::ColumnIndex(2),
                    debit: Field::ColumnIndex(3)
                },
                all_fields: hashmap! {
                    FieldKey::Date => Field::ColumnIndex(0),
                    FieldKey::Payee => Field::ColumnIndex(1),
                    FieldKey::Credit => Field::ColumnIndex(2),
                    FieldKey::Debit => Field::ColumnIndex(3),
                    FieldKey::Balance => Field::ColumnIndex(4),
                },
                max_column: 4,
            },
            got,
        );
    }

    #[test]
    fn field_map_try_new_index_amount() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Index(one_based!(1)),
            FieldKey::Payee => FieldPos::Index(one_based!(2)),
            FieldKey::Amount => FieldPos::Index(one_based!(3)),
        };
        let got = FieldMap::try_new(&config, &csv::StringRecord::from(vec!["unrelated"])).unwrap();
        assert_eq!(
            got,
            FieldMap {
                date: Field::ColumnIndex(0),
                payee: Field::ColumnIndex(1),
                value: TxnValueField::Amount(Field::ColumnIndex(2)),
                all_fields: hashmap! {
                    FieldKey::Date => Field::ColumnIndex(0),
                    FieldKey::Payee => Field::ColumnIndex(1),
                    FieldKey::Amount => Field::ColumnIndex(2),
                },
                max_column: 2,
            }
        );
    }

    #[test]
    fn field_map_try_new_optionals() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Index(one_based!(1)),
            FieldKey::Payee => FieldPos::Index(one_based!(2)),
            FieldKey::Amount => FieldPos::Index(one_based!(3)),
            FieldKey::Balance => FieldPos::Index(one_based!(4)),
            FieldKey::Category => FieldPos::Index(one_based!(5)),
            FieldKey::Commodity => FieldPos::Index(one_based!(6)),
            FieldKey::Rate => FieldPos::Index(one_based!(7)),
            FieldKey::SecondaryAmount => FieldPos::Index(one_based!(8)),
            FieldKey::SecondaryCommodity => FieldPos::Index(one_based!(9)),
        };
        let got = FieldMap::try_new(&config, &csv::StringRecord::from(vec!["unrelated"])).unwrap();
        assert_eq!(
            got,
            FieldMap {
                date: Field::ColumnIndex(0),
                payee: Field::ColumnIndex(1),
                value: TxnValueField::Amount(Field::ColumnIndex(2)),
                all_fields: hashmap! {
                    FieldKey::Date => Field::ColumnIndex(0),
                    FieldKey::Payee => Field::ColumnIndex(1),
                    FieldKey::Amount => Field::ColumnIndex(2),
                    FieldKey::Balance => Field::ColumnIndex(3),
                    FieldKey::Category => Field::ColumnIndex(4),
                    FieldKey::Commodity => Field::ColumnIndex(5),
                    FieldKey::Rate => Field::ColumnIndex(6),
                    FieldKey::SecondaryAmount => Field::ColumnIndex(7),
                    FieldKey::SecondaryCommodity => Field::ColumnIndex(8),
                },
                max_column: 8,
            }
        );
    }

    #[test]
    fn field_map_extract() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Index(one_based!(1)),
            FieldKey::Payee => FieldPos::Template(TemplateField { template: "{category} - {note}".parse().expect("this must be the correct template") }),
            FieldKey::Amount => FieldPos::Index(one_based!(2)),
            FieldKey::Category => FieldPos::Index(one_based!(3)),
            FieldKey::Note => FieldPos::Index(one_based!(4)),
        };
        let fm = FieldMap::try_new(
            &config,
            &csv::StringRecord::from(vec!["日付", "摘要", "お預け入れ額", "お引き出し額", "残高"]),
        )
        .expect("FieldMap::try_new must succeed");
        let record =
            csv::StringRecord::from(["2024/07/01", "123,456", "株買い", "東海旅客鉄道"].as_slice());
        let got = fm
            .extract(FieldKey::Date, &record)
            .expect("must not cause an error");
        assert_eq!("2024/07/01", got.expect("must be non-empty"));

        let got = fm
            .amount(AccountType::Asset, &record)
            .expect("fm.amount() must succeed");
        assert_eq!(dec!(123456), got);

        let got = fm
            .extract(FieldKey::Payee, &record)
            .expect("fm.extract(Payee, _) must succeed")
            .expect("fm.extract(Payee, _) must exists");
        assert_eq!("株買い - 東海旅客鉄道", got);
    }
}
