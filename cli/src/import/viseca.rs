pub mod format;
pub mod parser;

use super::amount::OwnedAmount;
use super::config;
use super::extract::{self};
use super::single_entry;
use super::single_entry::CommodityPair;
use super::ImportError;

pub fn import<R: std::io::Read>(
    r: R,
    config: &config::ConfigEntry,
) -> Result<Vec<single_entry::Txn>, ImportError> {
    let extractor = extract::Extractor::from_config(format::VisecaFormat, config)?;
    let mut parser =
        parser::Parser::new(std::io::BufReader::new(r), config.commodity.primary.clone());
    let mut result = Vec::new();
    while let Some(entry) = parser.parse_entry()? {
        let line_count = entry.line_count;
        let fragment = extractor.extract(entry.as_entity(&config.commodity.primary));
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
            txn.add_charge(fee.amount);
        }
        result.push(txn);
    }
    Ok(result)
}
