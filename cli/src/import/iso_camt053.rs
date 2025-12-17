pub(super) mod xmlnode;

use chrono::NaiveDate;
use either::Either;
use rust_decimal::Decimal;

use super::amount::OwnedAmount;
use super::config;
use super::extract::{self, CamtStrField, StrField};
use super::single_entry::{self, CommodityPair};
use super::ImportError;

impl xmlnode::Entry {
    /// Returns value_date if available, otherwise booking date.
    fn guess_value_date(&self) -> NaiveDate {
        self.value_date
            .as_ref()
            .unwrap_or(&self.booking_date)
            .as_naive_date()
    }
}

pub fn import<R>(r: R, config: &config::ConfigEntry) -> Result<Vec<single_entry::Txn>, ImportError>
where
    R: std::io::Read,
{
    let extractor = extract::Extractor::from_config(CamtFormat, &config)?;
    let mut buf = std::io::BufReader::new(r);
    let doc: xmlnode::Document = quick_xml::de::from_reader(&mut buf)?;
    let mut res = Vec::new();
    for (si, stmt) in doc.bank_to_customer.statements.into_iter().enumerate() {
        if let Some(opening_balance) = find_balance(&stmt, xmlnode::BalanceCode::Opening) {
            if let Some(first) = stmt.entries.first() {
                let mut txn = single_entry::Txn::new(
                    first.guess_value_date(),
                    "Initial Balance",
                    OwnedAmount {
                        commodity: opening_balance.commodity.clone(),
                        value: Decimal::ZERO,
                    },
                );
                txn.dest_account(single_entry::LABEL_ADJUSTMENTS);
                txn.balance(opening_balance);
                res.push(txn);
            }
        }
        let closing_balance = find_balance(&stmt, xmlnode::BalanceCode::Closing);
        let entries = match &config.format.row_order.unwrap_or_default() {
            config::RowOrder::OldToNew => Either::Left(stmt.entries.iter()),
            config::RowOrder::NewToOld => Either::Right(stmt.entries.iter().rev()),
        };
        for (ni, entry) in entries.enumerate() {
            if entry.details.transactions.is_empty() {
                // TODO(kikeg): Fix this code repetition.
                let amount = entry.amount.to_owned(entry.credit_or_debit.value);
                let fragment = extractor.extract(CamtEntity(entry, None));
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
                let mut txn = fragment.new_txn(entry.guess_value_date(), amount, || {
                    format!(".//BkToCstmrStmt/Stmt[{}]/Ntry[{}]", si + 1, ni + 1)
                });
                txn.effective_date(entry.booking_date.as_naive_date());
                add_charges(&mut txn, &entry.charges)?;
                res.push(txn);
            }
            for (di, transaction) in entry.details.transactions.iter().enumerate() {
                let amount = transaction
                    .amount
                    .to_owned(transaction.credit_or_debit.value);
                let fragment = extractor.extract(CamtEntity(entry, Some(transaction)));
                let code = transaction.refs.account_servicer_reference.clone();
                let mut txn = fragment.new_txn(entry.guess_value_date(), amount, || {
                    format!(
                        ".//BkToCstmrStmt/Stmt[{}]/Ntry[{}]/NtryDtls/TxDtls[{}] code={:?}",
                        si + 1,
                        ni + 1,
                        di + 1,
                        code,
                    )
                });
                txn.effective_date(entry.booking_date.as_naive_date())
                    .code_option(code);
                if let Some(detail_amount) = transaction
                    .amount_details
                    .as_ref()
                    .and_then(|x| x.transaction.as_ref())
                {
                    // TODO: Check this logic again so that it makes sense logically.
                    // https://github.com/xkikeg/okane/issues/289
                    // For now, we use transaciton amount, without falling back to instructed amount.
                    if transaction.amount != detail_amount.amount {
                        if let Some(exchange) = detail_amount.currency_exchange.as_ref() {
                            txn.add_rate(
                                CommodityPair {
                                    source: exchange.source_currency.clone(),
                                    target: exchange.target_currency.clone(),
                                },
                                exchange.exchange_rate.value,
                            )?;
                        }
                        txn.transferred_amount(
                            detail_amount
                                .amount
                                .to_owned(transaction.credit_or_debit.value),
                        );
                    }
                }
                add_charges(&mut txn, &entry.charges)?;
                add_charges(&mut txn, &transaction.charges)?;
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

fn find_balance(stmt: &xmlnode::Statement, code: xmlnode::BalanceCode) -> Option<OwnedAmount> {
    stmt.balance
        .iter()
        .filter(|x| x.balance_type.credit_or_property.code.value == code)
        .map(|x| x.amount.to_owned(x.credit_or_debit.value))
        .next()
}

fn add_charges(
    txn: &mut single_entry::Txn,
    charges: &Option<xmlnode::Charges>,
) -> Result<(), ImportError> {
    let charges = match charges {
        Some(charges) => charges,
        None => return Ok(()),
    };
    for cr in &charges.records {
        if cr.amount.value.is_zero() {
            continue;
        }
        log::info!("ADDED cr: {:?}", cr);
        // charge_amount must be negated, as charge is by default debit.
        let charge_amount = -cr.amount.to_owned(cr.credit_or_debit.value);
        if !cr.is_charge_included {
            txn.try_add_charge_not_included(charge_amount)?;
        } else {
            txn.add_charge(charge_amount);
        }
    }
    Ok(())
}

impl xmlnode::Amount {
    fn to_owned(&self, credit_or_debit: xmlnode::CreditOrDebit) -> OwnedAmount {
        OwnedAmount {
            value: match credit_or_debit {
                xmlnode::CreditOrDebit::Credit => self.value,
                xmlnode::CreditOrDebit::Debit => -self.value,
            },
            commodity: self.currency.clone(),
        }
    }
}

// Adapter for Extractor.

#[derive(Debug, Clone, Copy)]
struct CamtFormat;

impl extract::EntityFormat for CamtFormat {
    fn name(&self) -> &'static str {
        "ISO Camt053"
    }

    fn has_camt_transaction_code(&self) -> bool {
        true
    }

    fn has_str_field(&self, field: StrField) -> bool {
        match field {
            StrField::Camt(_) => true,
            StrField::Payee => false,
            StrField::Category => false,
            StrField::Commodity => true,
            StrField::SecondaryCommodity => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CamtEntity<'a>(&'a xmlnode::Entry, Option<&'a xmlnode::TransactionDetails>);

impl<'a> extract::Entity<'a> for CamtEntity<'a> {
    fn camt_transaction_code(&self) -> Option<&'a xmlnode::BankTransactionCode> {
        Some(&self.0.bank_transaction_code)
    }

    fn str_field(&self, field: StrField) -> Option<&'a str> {
        let CamtEntity(entry, transaction) = self;
        match field {
            StrField::Camt(fd) => match fd {
                CamtStrField::CreditorName => transaction
                    .and_then(|t| t.related_parties.as_ref())
                    .and_then(|rp| rp.creditor.as_ref())
                    .map(|cd| cd.name()),
                CamtStrField::CreditorAccountId => transaction
                    .and_then(|t| t.related_parties.as_ref())
                    .and_then(|rp| rp.creditor_account.as_ref())
                    .map(|a| a.id.value.as_str_id()),
                CamtStrField::UltimateCreditorName => transaction
                    .and_then(|t| t.related_parties.as_ref())
                    .and_then(|rp| rp.ultimate_creditor.as_ref())
                    .map(|ud| ud.name()),
                CamtStrField::DebtorName => transaction
                    .and_then(|t| t.related_parties.as_ref())
                    .and_then(|rp| rp.debtor.as_ref())
                    .map(|dt| dt.name()),
                CamtStrField::DebtorAccountId => transaction
                    .and_then(|t| t.related_parties.as_ref())
                    .and_then(|rp| rp.debtor_account.as_ref())
                    .map(|a| a.id.value.as_str_id()),
                CamtStrField::UltimateDebtorName => transaction
                    .and_then(|t| t.related_parties.as_ref())
                    .and_then(|rp| rp.ultimate_debtor.as_ref())
                    .map(|ud| ud.name()),
                CamtStrField::RemittanceUnstructuredInfo => transaction
                    .and_then(|t| t.remittance_info.as_ref())
                    .and_then(|i| i.unstructured.as_ref())
                    .map(|v| v.as_str()),
                CamtStrField::AdditionalEntryInfo => Some(&entry.additional_info),
                CamtStrField::AdditionalTransactionInfo => transaction
                    .and_then(|t| t.additional_info.as_ref())
                    .map(|ai| ai.as_str()),
            },
            StrField::Commodity => Some(entry.amount.currency.as_str()),
            StrField::SecondaryCommodity => transaction
                .as_ref()
                .and_then(|x| x.amount_details.as_ref())
                .and_then(|x| x.transaction.as_ref())
                .map(|x| x.amount.currency.as_str()),
            StrField::Payee | StrField::Category => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use extract::Entity;

    fn test_entry() -> xmlnode::Entry {
        xmlnode::Entry {
            amount: xmlnode::Amount {
                value: dec!(120),
                currency: "CHF".to_string(),
            },
            credit_or_debit: xmlnode::CreditDebitIndicator {
                value: xmlnode::CreditOrDebit::Credit,
            },
            booking_date: xmlnode::DateHolder::from_naive_date(
                NaiveDate::from_ymd_opt(2021, 10, 1).unwrap(),
            ),
            value_date: Some(xmlnode::DateHolder::from_naive_date(
                NaiveDate::from_ymd_opt(2021, 10, 1).unwrap(),
            )),
            bank_transaction_code: xmlnode::BankTransactionCode {
                domain: None,
                proprietary: None,
            },
            charges: None,
            additional_info: "entry additional info".to_string(),
            details: xmlnode::EntryDetails {
                batch: xmlnode::Batch {
                    number_of_transactions: 1,
                },
                transactions: vec![test_transaction()],
            },
        }
    }

    fn test_transaction() -> xmlnode::TransactionDetails {
        xmlnode::TransactionDetails {
            refs: xmlnode::References {
                account_servicer_reference: Some("foobar".to_string()),
            },
            credit_or_debit: xmlnode::CreditDebitIndicator {
                value: xmlnode::CreditOrDebit::Credit,
            },
            amount: xmlnode::Amount {
                value: dec!(12.3),
                currency: "CHF".to_string(),
            },
            amount_details: Some(xmlnode::AmountDetails {
                instructed: Some(xmlnode::AmountWithExchange {
                    amount: xmlnode::Amount {
                        value: dec!(12.3),
                        currency: "CHF".to_string(),
                    },
                    currency_exchange: None,
                }),
                transaction: Some(xmlnode::AmountWithExchange {
                    amount: xmlnode::Amount {
                        value: dec!(12.3),
                        currency: "CHF".to_string(),
                    },
                    currency_exchange: None,
                }),
            }),
            charges: None,
            related_parties: None,
            remittance_info: Some(xmlnode::RemittanceInfo {
                unstructured: Some("the remittance info".to_string()),
            }),
            additional_info: Some("This is additional Info".to_string()),
        }
    }

    #[test]
    fn camt_transaction_code_returns_value() {
        let code = xmlnode::BankTransactionCode {
            domain: Some(xmlnode::Domain {
                code: xmlnode::DomainCodeValue {
                    value: xmlnode::DomainCode::Payment,
                },
                family: xmlnode::DomainFamily {
                    code: xmlnode::DomainFamilyCodeValue {
                        value: xmlnode::DomainFamilyCode::IssuedCreditTransfers,
                    },
                    sub_family_code: xmlnode::DomainSubFamilyCodeValue {
                        value: xmlnode::DomainSubFamilyCode::AutomaticTransfer,
                    },
                },
            }),
            proprietary: None,
        };
        let entry = xmlnode::Entry {
            bank_transaction_code: code.clone(),
            ..test_entry()
        };
        let details = None;
        let entity = CamtEntity(&entry, details);

        assert_eq!(Some(&code), entity.camt_transaction_code());
    }

    mod str_field {
        use super::*;

        use pretty_assertions::assert_eq;
        use rstest::rstest;
        use rstest_reuse::{self, *};

        #[template]
        #[rstest]
        #[case(CamtStrField::CreditorName)]
        #[case(CamtStrField::CreditorAccountId)]
        #[case(CamtStrField::UltimateCreditorName)]
        #[case(CamtStrField::DebtorName)]
        #[case(CamtStrField::DebtorAccountId)]
        #[case(CamtStrField::UltimateDebtorName)]
        fn related_party_fields(#[case] field: CamtStrField) {}

        #[apply(related_party_fields)]
        fn related_party_fields_returns_none_without_detail(#[case] field: CamtStrField) {
            let entry = test_entry();
            let details = None;
            let entity = CamtEntity(&entry, details);

            assert_eq!(None, entity.str_field(StrField::Camt(field)));
        }

        #[apply(related_party_fields)]
        fn related_party_fields_returns_none_with_empty_detail(#[case] field: CamtStrField) {
            let entry = test_entry();
            let details = test_transaction();

            assert_eq!(
                None,
                CamtEntity(&entry, Some(&details)).str_field(StrField::Camt(field))
            );
        }

        #[apply(related_party_fields)]
        fn related_party_fields_returns_none_with_empty_related_parties(
            #[case] field: CamtStrField,
        ) {
            let entry = test_entry();
            let details = xmlnode::TransactionDetails {
                related_parties: Some(xmlnode::RelatedParties::default()),
                ..test_transaction()
            };

            assert_eq!(
                None,
                CamtEntity(&entry, Some(&details)).str_field(StrField::Camt(field))
            );
        }

        fn simple_related_party(x: &str) -> xmlnode::RelatedParty {
            xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                name: x.to_string(),
                ..xmlnode::PartyDetails::default()
            })
        }

        #[test]
        fn all_set() {
            let entry = test_entry();
            let details = xmlnode::TransactionDetails {
                related_parties: Some(xmlnode::RelatedParties {
                    creditor: Some(simple_related_party("creditor name")),
                    creditor_account: Some(xmlnode::Account::from_inner(xmlnode::AccountId::Iban(
                        "creditor-account-id".to_string(),
                    ))),
                    ultimate_creditor: Some(simple_related_party("ultimate creditor name")),
                    debtor: Some(simple_related_party("debtor name")),
                    debtor_account: Some(xmlnode::Account::from_inner(xmlnode::AccountId::Other(
                        xmlnode::OtherAccountId {
                            id: "debtor-account-id".to_string(),
                        },
                    ))),
                    ultimate_debtor: Some(simple_related_party("ultimate debtor name")),
                }),
                ..test_transaction()
            };
            let entity = CamtEntity(&entry, Some(&details));

            assert_eq!(
                Some("creditor name"),
                entity.str_field(StrField::Camt(CamtStrField::CreditorName))
            );
            assert_eq!(
                Some("ultimate creditor name"),
                entity.str_field(StrField::Camt(CamtStrField::UltimateCreditorName))
            );
            assert_eq!(
                Some("debtor-account-id"),
                entity.str_field(StrField::Camt(CamtStrField::DebtorAccountId))
            );
            assert_eq!(
                Some("debtor name"),
                entity.str_field(StrField::Camt(CamtStrField::DebtorName))
            );
            assert_eq!(
                Some("debtor name"),
                entity.str_field(StrField::Camt(CamtStrField::DebtorName))
            );
            assert_eq!(
                Some("debtor-account-id"),
                entity.str_field(StrField::Camt(CamtStrField::DebtorAccountId))
            );
            assert_eq!(Some("CHF"), entity.str_field(StrField::SecondaryCommodity));
        }

        #[rstest]
        #[case(CamtStrField::RemittanceUnstructuredInfo)]
        #[case(CamtStrField::AdditionalTransactionInfo)]
        fn remittance_info_without_details(#[case] field: CamtStrField) {
            let entry = test_entry();
            let details = xmlnode::TransactionDetails {
                remittance_info: Some(xmlnode::RemittanceInfo { unstructured: None }),
                additional_info: None,
                ..test_transaction()
            };

            assert_eq!(
                None,
                CamtEntity(&entry, None).str_field(StrField::Camt(field))
            );
            assert_eq!(
                None,
                CamtEntity(&entry, Some(&details)).str_field(StrField::Camt(field))
            );
        }

        #[test]
        fn remittance_info_filled() {
            let entry = xmlnode::Entry {
                additional_info: "additional entry info".to_string(),
                ..test_entry()
            };
            let details = xmlnode::TransactionDetails {
                remittance_info: Some(xmlnode::RemittanceInfo {
                    unstructured: Some("remittance info".to_string()),
                }),
                additional_info: Some("additional transaction info".to_string()),
                ..test_transaction()
            };

            let entity = CamtEntity(&entry, Some(&details));

            assert_eq!(
                Some("remittance info"),
                entity.str_field(StrField::Camt(CamtStrField::RemittanceUnstructuredInfo))
            );
            assert_eq!(
                Some("additional entry info"),
                entity.str_field(StrField::Camt(CamtStrField::AdditionalEntryInfo))
            );

            assert_eq!(
                Some("additional transaction info"),
                entity.str_field(StrField::Camt(CamtStrField::AdditionalTransactionInfo))
            );
        }
    }
}
