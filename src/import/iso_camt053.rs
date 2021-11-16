mod extract;
mod xmlnode;

use super::config;
use super::single_entry;
use super::ImportError;
use crate::data;

use rust_decimal::Decimal;

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
            if let Some(opening_balance) = find_balance(&stmt, xmlnode::BalanceCode::Opening) {
                if let Some(first) = stmt.entries.first() {
                    let mut txn = single_entry::Txn::new(
                        first.value_date.date,
                        "Initial Balance",
                        data::Amount {
                            commodity: opening_balance.commodity.clone(),
                            value: Decimal::ZERO,
                        },
                    );
                    txn.dest_account("Equity:Adjustments");
                    txn.balance(opening_balance);
                    res.push(txn);
                }
            }
            let closing_balance = find_balance(&stmt, xmlnode::BalanceCode::Closing);
            for entry in stmt.entries {
                if entry.details.transactions.is_empty() {
                    // TODO(kikeg): Fix this code repetition.
                    let amount = entry.amount.to_data(entry.credit_or_debit.value);
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
                    if !fragment.cleared {
                        txn.clear_state(data::ClearState::Pending);
                    }
                    add_charges(&mut txn, config, &entry.charges)?;
                    res.push(txn);
                }
                for transaction in &entry.details.transactions {
                    let amount = transaction
                        .amount
                        .to_data(transaction.credit_or_debit.value);
                    let fragment = extractor.extract(&entry, Some(transaction));
                    let code = transaction
                        .refs
                        .account_servicer_reference.as_deref();
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
                        .code_option(code)
                        .dest_account_option(fragment.account);
                    if !fragment.cleared {
                        txn.clear_state(data::ClearState::Pending);
                    }
                    if let Some(amount_details) = transaction.amount_details.as_ref() {
                        if transaction.amount != amount_details.transaction.amount {
                            txn.transferred_amount(data::ExchangedAmount {
                                amount: amount_details
                                    .transaction
                                    .amount
                                    .to_data(transaction.credit_or_debit.value),
                                exchange: amount_details
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
                    }
                    add_charges(&mut txn, config, &entry.charges)?;
                    add_charges(&mut txn, config, &transaction.charges)?;
                    res.push(txn);
                }
            }
            if let Some(last_txn) = res.last_mut() {
                if let Some(b) = closing_balance {
                    last_txn.balance(b);
                }
            }
        }
        Ok(res)
    }
}

fn find_balance(stmt: &xmlnode::Statement, code: xmlnode::BalanceCode) -> Option<data::Amount> {
    stmt.balance
        .iter()
        .filter(|x| x.balance_type.credit_or_property.code.value == code)
        .map(|x| x.amount.to_data(x.credit_or_debit.value))
        .next()
}

fn add_charges(
    txn: &mut single_entry::Txn,
    config: &config::ConfigEntry,
    charges: &Option<xmlnode::Charges>,
) -> Result<(), ImportError> {
    if let Some(charges) = &charges {
        for cr in &charges.records {
            if cr.amount.value.is_zero() {
                continue;
            }
            if !cr.is_charge_included {
                return Err(ImportError::Unimplemented("ChrgInclInd=false unsupported"));
            }
            let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
                "config should have operator to have charge",
            ))?;
            txn.add_charge(payee, cr.amount.to_data(cr.credit_or_debit.value));
        }
    }
    Ok(())
}

impl xmlnode::Amount {
    fn to_data(&self, credit_or_debit: xmlnode::CreditOrDebit) -> data::Amount {
        data::Amount {
            value: match credit_or_debit {
                xmlnode::CreditOrDebit::Credit => self.value,
                xmlnode::CreditOrDebit::Debit => -self.value,
            },
            commodity: self.currency.clone(),
        }
    }
}
