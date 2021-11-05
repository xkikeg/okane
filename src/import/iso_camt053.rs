mod extract;
mod xmlnode;

use super::config;
use super::single_entry;
use super::ImportError;
use crate::data;

pub struct ISOCamt053Importer {}

impl super::Importer for ISOCamt053Importer {
    fn import<R>(
        &self,
        r: &mut R,
        config: &config::ConfigEntry,
    ) -> Result<Vec<single_entry::Txn>, ImportError>
    where
        R: std::io::Read,
    {
        let extractor = extract::from_config(config)?;
        let mut buf = std::io::BufReader::new(r);
        let doc: xmlnode::Document = quick_xml::de::from_reader(&mut buf)?;
        let mut res = Vec::new();
        for stmt in doc.bank_to_customer.statements {
            for entry in stmt.entries {
                if entry.charges.is_some() && entry.details.batch.number_of_transactions > 1 {
                    return Err(ImportError::Unimplemented(
                        "charge with multi batch transaction isn't supported",
                    ));
                }
                if entry.details.transactions.is_empty() {
                    // TODO(kikeg): Fix this code repetition.
                    let amount = to_data_amount(entry.credit_or_debit.value, &entry.amount);
                    let fragment = extractor.extract(&entry, None);
                    if fragment.payee.is_none() {
                        log::warn!("payee not set @ {:?} {:?}", entry.booking_date, amount);
                    } else if fragment.account.is_none() {
                        log::warn!(
                            "account not set @ {:?} {:?} {}",
                            entry.booking_date,
                            amount,
                            fragment.payee.unwrap()
                        );
                    }
                    let mut txn = single_entry::Txn::new(
                        entry.value_date.date,
                        fragment.payee.unwrap_or("unknown payee"),
                        amount,
                    );
                    txn.effective_date(entry.booking_date.date)
                        .dest_account_option(fragment.account);
                    res.push(txn);
                }
                for transaction in &entry.details.transactions {
                    let amount =
                        to_data_amount(transaction.credit_or_debit.value, &transaction.amount);
                    let fragment = extractor.extract(&entry, Some(transaction));
                    let code = transaction.refs.account_servicer_reference.as_str();
                    if fragment.payee.is_none() {
                        log::warn!("payee not set @ {:?}", code);
                    } else if fragment.account.is_none() {
                        log::warn!("account not set @ {:?} {}", code, fragment.payee.unwrap());
                    }
                    let mut txn = single_entry::Txn::new(
                        entry.value_date.date,
                        fragment.payee.unwrap_or("unknown payee"),
                        amount,
                    );
                    txn.effective_date(entry.booking_date.date)
                        .code(code)
                        .dest_account_option(fragment.account);
                    if transaction.amount != transaction.amount_details.transaction.amount {
                        txn.transferred_amount(data::ExchangedAmount {
                            amount: to_data_amount(
                                transaction.credit_or_debit.value,
                                &transaction.amount_details.transaction.amount,
                            ),
                            exchange: transaction
                                .amount_details
                                .transaction
                                .currency_exchange
                                .as_ref()
                                .map(|x| {
                                    data::Exchange::Rate(data::Amount {
                                        value: x.exchange_rate.value,
                                        commodity: x.source_currency.clone(),
                                    })
                                }),
                        });
                    }
                    res.push(txn);
                }
            }
        }
        Ok(res)
    }
}

fn to_data_amount(
    credit_or_debit: xmlnode::CreditOrDebit,
    amount: &xmlnode::Amount,
) -> data::Amount {
    data::Amount {
        value: match credit_or_debit {
            xmlnode::CreditOrDebit::Credit => amount.value,
            xmlnode::CreditOrDebit::Debit => -amount.value,
        },
        commodity: amount.currency.clone(),
    }
}
