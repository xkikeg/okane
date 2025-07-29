//! Defines CSV field utilities.

use std::{borrow::Cow, collections::HashMap};

use rust_decimal::Decimal;

use super::utility::str_to_comma_decimal;
use crate::import::{
    config::{self, FieldKey},
    template::{self, Template, TemplateKey},
    ImportError,
};

#[derive(Debug, PartialEq)]
enum AmountField {
    CreditDebit { credit: Field, debit: Field },
    Absolute(Field),
}

#[derive(Debug, PartialEq, Clone)]
enum Field {
    ColumnIndex(usize),
    Template(Template),
}

struct MappedRecord<'a> {
    field_resolver: &'a FieldResolver,
    record: &'a csv::StringRecord,
}

// here we only supports 32-bits or more bits system.
const _: () = assert!(u32::BITS <= usize::BITS);

impl<'a> template::Interpolate<'a> for MappedRecord<'a> {
    fn interpolate(&self, key: TemplateKey) -> Option<&'a str> {
        let key = match key {
            // Clone here might have penalty on template,
            // but it will be a failure path so good to go.
            TemplateKey::Named(fk) => self.field_resolver.field(fk)?.clone(),
            TemplateKey::Indexed(i) => Field::ColumnIndex(i.as_zero_based().try_into().unwrap()),
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
pub struct FieldResolver {
    date: Field,
    payee: Field,
    value: AmountField,
    all_fields: HashMap<FieldKey, Field>,
    max_column: usize,
}

impl FieldResolver {
    pub fn max(&self) -> usize {
        self.max_column
    }

    fn field(&self, field_key: FieldKey) -> Option<&Field> {
        self.all_fields.get(&field_key)
    }

    pub fn extract<'a>(
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
                        field_resolver: self,
                        record,
                    },
                )
                .map(|x| Some(Cow::Owned(format!("{}", x)))),
        }
    }

    pub fn amount(
        &self,
        at: config::AccountType,
        r: &csv::StringRecord,
    ) -> Result<Decimal, ImportError> {
        match &self.value {
            AmountField::CreditDebit { credit, debit } => {
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
            AmountField::Absolute(a) => {
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

    pub fn try_new(
        config_mapping: &HashMap<config::FieldKey, config::FieldPos>,
        header: &csv::StringRecord,
    ) -> Result<Self, ImportError> {
        let hm: HashMap<&str, usize> = header.iter().enumerate().map(|(k, v)| (v, k)).collect();
        let mut not_found_labels: Vec<&str> = Vec::new();
        let mut ki: HashMap<config::FieldKey, Field> = HashMap::with_capacity(config_mapping.len());
        for (&k, pos) in config_mapping {
            let field = match &pos {
                config::FieldPos::Index(i) => {
                    Some(Field::ColumnIndex(i.as_zero_based().try_into().unwrap()))
                }
                config::FieldPos::Label(label) => match hm.get(label.as_str()).cloned() {
                    Some(i) => Some(Field::ColumnIndex(i)),
                    None => {
                        // Report the error later to collect all not found labels.
                        not_found_labels.push(label);
                        None
                    }
                },
                config::FieldPos::Template(config::TemplateField { template }) => {
                    let template = template.parse()?;
                    Some(Field::Template(template))
                }
            };
            if let Some(field) = field {
                ki.insert(k, field);
            }
        }
        if !not_found_labels.is_empty() {
            let mut actual_labels: Vec<&str> = hm.keys().cloned().collect();
            actual_labels.sort_unstable();
            not_found_labels.sort_unstable();
            return Err(ImportError::Other(format!(
                "specified labels not found: {} actual labels: {}",
                not_found_labels.join(","),
                actual_labels.join(",")
            )));
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
            Some(a) => Ok(AmountField::Absolute(a)),
            None => credit
                .zip(debit)
                .map(|(c, d)| AmountField::CreditDebit {
                    credit: c,
                    debit: d,
                })
                .ok_or(ImportError::InvalidConfig(
                    "either amount or credit/debit pair should be set",
                )),
        }?;
        Ok(FieldResolver {
            date,
            payee,
            value,
            all_fields: ki,
            max_column,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::config::{AccountType, FieldPos};
    use crate::one_based_macro::one_based_32;

    #[test]
    fn field_map_try_new_label_credit_debit() {
        let config: HashMap<FieldKey, FieldPos> = hashmap! {
            FieldKey::Date => FieldPos::Label("日付".to_owned()),
            FieldKey::Payee => FieldPos::Label("摘要".to_owned()),
            FieldKey::Credit => FieldPos::Label("お預け入れ額".to_owned()),
            FieldKey::Debit => FieldPos::Label("お引き出し額".to_owned()),
            FieldKey::Balance => FieldPos::Label("残高".to_owned()),
        };
        let got = FieldResolver::try_new(
            &config,
            &csv::StringRecord::from(vec!["日付", "摘要", "お預け入れ額", "お引き出し額", "残高"]),
        )
        .unwrap();
        assert_eq!(
            FieldResolver {
                date: Field::ColumnIndex(0),
                payee: Field::ColumnIndex(1),
                value: AmountField::CreditDebit {
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
            FieldKey::Date => FieldPos::Index(one_based_32!(1)),
            FieldKey::Payee => FieldPos::Index(one_based_32!(2)),
            FieldKey::Amount => FieldPos::Index(one_based_32!(3)),
        };
        let got =
            FieldResolver::try_new(&config, &csv::StringRecord::from(vec!["unrelated"])).unwrap();
        assert_eq!(
            got,
            FieldResolver {
                date: Field::ColumnIndex(0),
                payee: Field::ColumnIndex(1),
                value: AmountField::Absolute(Field::ColumnIndex(2)),
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
            FieldKey::Date => FieldPos::Index(one_based_32!(1)),
            FieldKey::Payee => FieldPos::Index(one_based_32!(2)),
            FieldKey::Amount => FieldPos::Index(one_based_32!(3)),
            FieldKey::Balance => FieldPos::Index(one_based_32!(4)),
            FieldKey::Category => FieldPos::Index(one_based_32!(5)),
            FieldKey::Commodity => FieldPos::Index(one_based_32!(6)),
            FieldKey::Rate => FieldPos::Index(one_based_32!(7)),
            FieldKey::SecondaryAmount => FieldPos::Index(one_based_32!(8)),
            FieldKey::SecondaryCommodity => FieldPos::Index(one_based_32!(9)),
        };
        let got =
            FieldResolver::try_new(&config, &csv::StringRecord::from(vec!["unrelated"])).unwrap();
        assert_eq!(
            got,
            FieldResolver {
                date: Field::ColumnIndex(0),
                payee: Field::ColumnIndex(1),
                value: AmountField::Absolute(Field::ColumnIndex(2)),
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
            FieldKey::Date => FieldPos::Index(one_based_32!(1)),
            FieldKey::Payee => FieldPos::Template(config::TemplateField { template: "{category} - {note}".parse().expect("this must be the correct template") }),
            FieldKey::Amount => FieldPos::Index(one_based_32!(2)),
            FieldKey::Category => FieldPos::Index(one_based_32!(3)),
            FieldKey::Note => FieldPos::Index(one_based_32!(4)),
        };
        let fm = FieldResolver::try_new(
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
