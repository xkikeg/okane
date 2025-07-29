mod field;
mod matcher;
mod utility;

use std::borrow::Cow;
use std::convert::TryInto;
use std::io::BufRead;
use std::io::BufReader;

use chrono::NaiveDate;

use okane_core::syntax;

use super::amount::OwnedAmount;
use super::config::{self, FieldKey};
use super::extract;
use super::single_entry::{self, CommodityPair};
use super::ImportError;
use utility::str_to_comma_decimal;

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
    let skip_config = &config.format.skip.unwrap_or_default();
    for i in 0..skip_config.head {
        let mut skipped = String::new();
        br.read_line(&mut skipped)?;
        log::info!("skipped {}-th line: {}", i, skipped.as_str().trim_end());
    }
    let mut rdr = rb.from_reader(br);
    let header = rdr.headers()?;
    let resolver = field::FieldResolver::try_new(&config.format.fields, header)?;
    let extractor: extract::Extractor<matcher::CsvMatcher> = (&config.rewrite).try_into()?;
    let default_conversion: &config::CommodityConversionSpec = &config.commodity.conversion;
    for record in rdr.records() {
        if let Some(txn) =
            extract_transaction(config, &resolver, &extractor, default_conversion, &record?)?
        {
            res.push(txn);
        }
    }
    match config.format.row_order.unwrap_or_default() {
        config::RowOrder::OldToNew => (),
        config::RowOrder::NewToOld => {
            res.reverse();
        }
    }
    Ok(res)
}

fn extract_transaction(
    config: &config::ConfigEntry,
    resolver: &field::FieldResolver,
    extractor: &extract::Extractor<matcher::CsvMatcher>,
    default_conversion: &config::CommodityConversionSpec,
    r: &csv::StringRecord,
) -> Result<Option<single_entry::Txn>, ImportError> {
    let pos = r.position().expect("csv record position");
    if r.len() <= resolver.max() {
        return Err(ImportError::Other(format!(
            "csv record length too short at line {}: want {}, got {}",
            pos.line(),
            resolver.max(),
            r.len()
        )));
    }
    let datestr = resolver
        .extract(FieldKey::Date, r)?
        .ok_or_else(|| ImportError::Other("Field date must be present".to_string()))?;
    if datestr.is_empty() {
        log::info!("skip empty date at line {}", pos.line());
        return Ok(None);
    }
    let date = NaiveDate::parse_from_str(&datestr, config.format.date.as_str())?;
    let original_payee = resolver
        .extract(FieldKey::Payee, r)?
        .ok_or_else(|| ImportError::Other("Field payee must be present".to_string()))?;
    let amount = resolver.amount(config.account_type, r)?;
    let balance = resolver
        .extract(FieldKey::Balance, r)?
        .map_or(Ok(None), |s| str_to_comma_decimal(&s))?;
    let secondary_amount = resolver
        .extract(FieldKey::SecondaryAmount, r)?
        .map_or(Ok(None), |s| str_to_comma_decimal(&s))?;
    let secondary_commodity = resolver.extract(FieldKey::SecondaryCommodity, r)?;
    let category = resolver.extract(FieldKey::Category, r)?;
    let commodity = resolver
        .extract(FieldKey::Commodity, r)?
        .unwrap_or_else(|| Cow::Borrowed(&config.commodity.primary));
    let rate = resolver
        .extract(FieldKey::Rate, r)?
        .map_or_else(|| Ok(None), |s| str_to_comma_decimal(&s))?;
    let fragment = extractor.extract(matcher::Record {
        payee: &original_payee,
        category: category.as_deref(),
        secondary_commodity: secondary_commodity.as_deref(),
    });
    let payee = fragment.payee.unwrap_or(&original_payee);
    if fragment.account.is_none() {
        log::warn!("account unmatched at line {}, payee={}", pos.line(), payee);
    }
    let mut txn = single_entry::Txn::new(
        date,
        payee,
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
    if let Some(note) = resolver.extract(FieldKey::Note, r)? {
        if !note.trim().is_empty() {
            txn.add_comment(note.into_owned());
        }
    }
    if let Some(b) = balance {
        txn.balance(OwnedAmount {
            value: b,
            commodity: commodity.clone().into_owned(),
        });
    }
    if let Some(charge) = resolver.extract(FieldKey::Charge, r)? {
        let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
            "config should have operator to have charge",
        ))?;
        match str_to_comma_decimal(&charge)? {
            Some(value) if !value.is_zero() => {
                txn.add_charge(
                    payee,
                    OwnedAmount {
                        value,
                        commodity: commodity.clone().into_owned(),
                    },
                );
            }
            _ => (),
        }
    }
    let default_conversion =
        if rate.is_some() && secondary_amount.is_some() && secondary_commodity.is_some() {
            Some(default_conversion)
        } else {
            None
        };
    let conversion = fragment
        .conversion
        .or(default_conversion)
        .filter(|x| !x.disabled.unwrap_or_default());
    if let Some(conv) = conversion {
        let rate = rate.ok_or_else(|| {
            ImportError::Other(format!(
                "no rate specified for transcation with conversion: line {}",
                pos.line()
            ))
        })?;
        let secondary_commodity = conv.commodity.as_deref().or(secondary_commodity.as_deref())
                .ok_or_else(||ImportError::Other(format!("either rewrite.conversion.commodity or secondary_commodity field must be set @ line {}", pos.line())))?;
        let (rate_key, computed_transferred) = match conv.rate.unwrap_or_default() {
            config::ConversionRateMode::PriceOfPrimary => (
                CommodityPair {
                    source: secondary_commodity.to_owned(),
                    target: commodity.into_owned(),
                },
                amount * rate,
            ),
            config::ConversionRateMode::PriceOfSecondary => (
                CommodityPair {
                    source: commodity.into_owned(),
                    target: secondary_commodity.to_owned(),
                },
                amount / rate,
            ),
        };
        txn.add_rate(rate_key, rate)?;
        let transferred = match conv.amount.unwrap_or_default() {
                    config::ConversionAmountMode::Extract => secondary_amount.ok_or_else(|| ImportError::Other(format!(
                            "secondary_amount should be specified when conversion.amount is set to extract @ line {}", pos.line()
                    )))?,
                    config::ConversionAmountMode::Compute => computed_transferred,
                };
        txn.transferred_amount(OwnedAmount {
            value: transferred,
            commodity: secondary_commodity.to_owned(),
        });
    }
    Ok(Some(txn))
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use maplit::hashmap;

    use config::FieldPos;

    use crate::one_based::OneBasedIndex;

    fn empty_config() -> config::ConfigEntry {
        config::ConfigEntry {
            path: "/not/used".to_string(),
            encoding: config::Encoding(encoding_rs::UTF_8),
            account: "Assets:Bank".to_string(),
            account_type: config::AccountType::Asset,
            operator: None,
            commodity: config::AccountCommoditySpec::default(),
            format: config::FormatSpec::default(),
            output: config::OutputSpec::default(),
            rewrite: Vec::new(),
        }
    }

    #[test]
    fn import_fails_if_columns_missing() {
        let input = indoc! {"
            date,payee,amount
            2025/01/02,foo
        "};
        let err = import(
            input.as_bytes(),
            &config::ConfigEntry {
                format: config::FormatSpec {
                    date: "%Y/%m/%d".to_string(),
                    fields: hashmap! {
                        FieldKey::Date => FieldPos::Index(OneBasedIndex::from_one_based(1).unwrap()),
                        FieldKey::Payee => FieldPos::Index(OneBasedIndex::from_one_based(2).unwrap()),
                        FieldKey::Amount => FieldPos::Index(OneBasedIndex::from_one_based(3).unwrap()),
                    },
                    delimiter: "".to_string(),
                    skip: None,
                    row_order: None,
                },
                ..empty_config()
            },
        )
        .unwrap_err();
        assert!(
            matches!(&err, ImportError::Other(s) if s.contains("csv record length too short")),
            "unexpected error: {err}"
        );
    }
}
