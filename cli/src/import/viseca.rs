pub mod format;
pub mod parser;

use super::amount::OwnedAmount;
use super::config;
use super::extract::{self, StrField};
use super::single_entry;
use super::single_entry::CommodityPair;
use super::ImportError;

pub fn import<R: std::io::Read>(
    r: R,
    config: &config::ConfigEntry,
) -> Result<Vec<single_entry::Txn>, ImportError> {
    let extractor = extract::Extractor::try_new(&config.rewrite, VisecaFormat)?;
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
struct VisecaFormat;

impl extract::EntityFormat for VisecaFormat {
    fn name(&self) -> &'static str {
        "viseca"
    }

    fn has_camt_transaction_code(&self) -> bool {
        false
    }

    fn has_str_field(&self, field: StrField) -> bool {
        match field {
            StrField::Camt(_) => false,
            StrField::Payee => true,
            StrField::Category => true,
            StrField::SecondaryCommodity => true,
        }
    }
}
