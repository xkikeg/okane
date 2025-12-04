mod xmlnode;

use std::convert::{TryFrom, TryInto};

use chrono::NaiveDate;
use either::Either;
use regex::Regex;
use rust_decimal::Decimal;

use okane_core::syntax;

use super::amount::OwnedAmount;
use super::config;
use super::extract;
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
    let extractor: extract::Extractor<FieldMatch> = (&config.rewrite).try_into()?;
    let mut buf = std::io::BufReader::new(r);
    let doc: xmlnode::Document = quick_xml::de::from_reader(&mut buf)?;
    let mut res = Vec::new();
    for stmt in doc.bank_to_customer.statements {
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
        for entry in entries {
            if entry.details.transactions.is_empty() {
                // TODO(kikeg): Fix this code repetition.
                let amount = entry.amount.to_owned(entry.credit_or_debit.value);
                let fragment = extractor.extract((entry, None));
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
                    entry.guess_value_date(),
                    fragment.payee.unwrap_or("unknown payee"),
                    amount,
                );
                txn.effective_date(entry.booking_date.as_naive_date())
                    .dest_account_option(fragment.account);
                if !fragment.cleared {
                    txn.clear_state(syntax::ClearState::Pending);
                }
                add_charges(&mut txn, config, &entry.charges)?;
                res.push(txn);
            }
            for transaction in &entry.details.transactions {
                let amount = transaction
                    .amount
                    .to_owned(transaction.credit_or_debit.value);
                let fragment = extractor.extract((entry, Some(transaction)));
                let code = transaction.refs.account_servicer_reference.clone();
                if fragment.payee.is_none() {
                    log::warn!("payee not set @ {:?}", code);
                } else if fragment.account.is_none() {
                    log::warn!("account not set @ {:?} {}", code, fragment.payee.unwrap());
                }
                let mut txn = single_entry::Txn::new(
                    entry.guess_value_date(),
                    fragment.payee.unwrap_or("unknown payee"),
                    amount,
                );
                txn.effective_date(entry.booking_date.as_naive_date())
                    .code_option(code)
                    .dest_account_option(fragment.account);
                if !fragment.cleared {
                    txn.clear_state(syntax::ClearState::Pending);
                }
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

fn find_balance(stmt: &xmlnode::Statement, code: xmlnode::BalanceCode) -> Option<OwnedAmount> {
    stmt.balance
        .iter()
        .filter(|x| x.balance_type.credit_or_property.code.value == code)
        .map(|x| x.amount.to_owned(x.credit_or_debit.value))
        .next()
}

fn add_charges(
    txn: &mut single_entry::Txn,
    config: &config::ConfigEntry,
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
        let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
            "config should have operator to have charge",
        ))?;
        log::info!("ADDED cr: {:?}", cr);
        // charge_amount must be negated, as charge is by default debit.
        let charge_amount = -cr.amount.to_owned(cr.credit_or_debit.value);
        if !cr.is_charge_included {
            txn.try_add_charge_not_included(payee, charge_amount)?;
        } else {
            txn.add_charge(payee, charge_amount);
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

#[derive(Debug)]
enum FieldMatch {
    DomainCode(xmlnode::DomainCode),
    DomainFamily(xmlnode::DomainFamilyCode),
    DomainSubFamily(xmlnode::DomainSubFamilyCode),
    RegexMatch(MatchField, Regex),
}

#[derive(Debug, PartialEq)]
enum MatchField {
    CreditorName,
    CreditorAccountId,
    UltimateCreditorName,
    DebtorName,
    DebtorAccountId,
    UltimateDebtorName,
    RemittanceUnstructuredInfo,
    AdditionalEntryInfo,
    AdditionalTransactionInfo,
    Payee,
}

fn to_field(f: config::RewriteField) -> Result<MatchField, ImportError> {
    match f {
        config::RewriteField::CreditorName => Some(MatchField::CreditorName),
        config::RewriteField::CreditorAccountId => Some(MatchField::CreditorAccountId),
        config::RewriteField::UltimateCreditorName => Some(MatchField::UltimateCreditorName),
        config::RewriteField::DebtorName => Some(MatchField::DebtorName),
        config::RewriteField::DebtorAccountId => Some(MatchField::DebtorAccountId),
        config::RewriteField::UltimateDebtorName => Some(MatchField::UltimateDebtorName),
        config::RewriteField::RemittanceUnstructuredInfo => {
            Some(MatchField::RemittanceUnstructuredInfo)
        }
        config::RewriteField::AdditionalEntryInfo => Some(MatchField::AdditionalEntryInfo),
        config::RewriteField::AdditionalTransactionInfo => {
            Some(MatchField::AdditionalTransactionInfo)
        }
        config::RewriteField::Payee => Some(MatchField::Payee),
        _ => None,
    }
    .ok_or_else(|| ImportError::Other(format!("unknown match field: {:?}", f)))
}

impl TryFrom<(config::RewriteField, &str)> for FieldMatch {
    type Error = ImportError;
    fn try_from((f, v): (config::RewriteField, &str)) -> Result<FieldMatch, ImportError> {
        Ok(match f {
            config::RewriteField::DomainCode => {
                let code = serde_yaml::from_str(v)?;
                FieldMatch::DomainCode(code)
            }
            config::RewriteField::DomainFamily => {
                let code = serde_yaml::from_str(v)?;
                FieldMatch::DomainFamily(code)
            }
            config::RewriteField::DomainSubFamily => {
                let code = serde_yaml::from_str(v)?;
                FieldMatch::DomainSubFamily(code)
            }
            _ => {
                let pattern = extract::regex_matcher(v)?;
                let field = to_field(f)?;
                FieldMatch::RegexMatch(field, pattern)
            }
        })
    }
}

impl<'a> extract::Entity<'a> for FieldMatch {
    type T = (&'a xmlnode::Entry, Option<&'a xmlnode::TransactionDetails>);
}

impl extract::EntityMatcher for FieldMatch {
    fn captures<'a>(
        &self,
        fragment: &extract::Fragment<'a>,
        (entry, transaction): (&'a xmlnode::Entry, Option<&'a xmlnode::TransactionDetails>),
    ) -> Option<extract::Matched<'a>> {
        let has_match = match self {
            FieldMatch::DomainCode(code) => {
                Either::Left(*code == entry.bank_transaction_code.domain_code()?)
            }
            FieldMatch::DomainFamily(code) => {
                Either::Left(*code == entry.bank_transaction_code.domain_family_code()?)
            }
            FieldMatch::DomainSubFamily(code) => {
                Either::Left(*code == entry.bank_transaction_code.domain_sub_family_code()?)
            }
            FieldMatch::RegexMatch(fd, re) => {
                let target: Option<&str> = match fd {
                    MatchField::CreditorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.creditor.as_ref())
                        .map(|cd| cd.name()),
                    MatchField::CreditorAccountId => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.creditor_account.as_ref())
                        .map(|a| a.id.value.as_str_id()),
                    MatchField::UltimateCreditorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.ultimate_creditor.as_ref())
                        .map(|ud| ud.name()),
                    MatchField::DebtorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.debtor.as_ref())
                        .map(|dt| dt.name()),
                    MatchField::DebtorAccountId => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.debtor_account.as_ref())
                        .map(|a| a.id.value.as_str_id()),
                    MatchField::UltimateDebtorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.ultimate_debtor.as_ref())
                        .map(|ud| ud.name()),
                    MatchField::RemittanceUnstructuredInfo => transaction
                        .and_then(|t| t.remittance_info.as_ref())
                        .and_then(|i| i.unstructured.as_ref())
                        .map(|v| v.as_str()),
                    MatchField::AdditionalEntryInfo => Some(&entry.additional_info),
                    MatchField::AdditionalTransactionInfo => transaction
                        .and_then(|t| t.additional_info.as_ref())
                        .map(|ai| ai.as_str()),
                    MatchField::Payee => fragment.payee,
                };
                Either::Right(target.and_then(|t| re.captures(t)).map(|c| c.into()))
            }
        };
        has_match.right_or_else(|matched| {
            if matched {
                Some(extract::Matched::default())
            } else {
                None
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

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
            related_parties: Some(xmlnode::RelatedParties {
                debtor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "debtor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                creditor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "creditor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                creditor_account: None,
                debtor_account: None,
                ultimate_debtor: None,
                ultimate_creditor: None,
            }),
            remittance_info: Some(xmlnode::RemittanceInfo {
                unstructured: Some("the remittance info".to_string()),
            }),
            additional_info: Some("This is additional Info".to_string()),
        }
    }

    use extract::{EntityMatcher, Fragment, Matched};

    #[test]
    fn field_match_from_invalid_domain_family() {
        let err = FieldMatch::try_from((config::RewriteField::DomainCode, "foo")).unwrap_err();
        match err {
            ImportError::YAML(cause) => {
                assert!(
                    cause.to_string().contains("unknown variant `foo`"),
                    "{:?} did not contains expected error",
                    cause
                );
            }
            _ => {
                panic!("unexpected type of error: {:?}", err);
            }
        }
    }

    #[test]
    fn field_match_captures_domain_family_match() {
        let m = FieldMatch::try_from((config::RewriteField::DomainFamily, "ICDT")).unwrap();
        let entry = test_entry();

        let got = m.captures(&Fragment::default(), (&entry, None));

        assert_eq!(Some(Matched::default()), got);
    }

    #[test]
    fn field_match_captures_domain_family_unmatch() {
        let m = FieldMatch::try_from((config::RewriteField::DomainFamily, "RCDT")).unwrap();
        let entry = test_entry();

        let got = m.captures(&Fragment::default(), (&entry, None));

        assert_eq!(None, got);
    }
    #[test]
    fn field_match_captures_domain_sub_family_match() {
        let m = FieldMatch::try_from((config::RewriteField::DomainSubFamily, "AUTT")).unwrap();
        let entry = test_entry();

        let got = m.captures(&Fragment::default(), (&entry, None));

        assert_eq!(Some(Matched::default()), got);
    }

    #[test]
    fn field_match_captures_domain_sub_family_unmatch() {
        let m = FieldMatch::try_from((config::RewriteField::DomainSubFamily, "SALA")).unwrap();
        let entry = test_entry();

        let got = m.captures(&Fragment::default(), (&entry, None));

        assert_eq!(None, got);
    }

    #[test]
    fn field_match_from_invalid_regex() {
        let err = FieldMatch::try_from((config::RewriteField::Payee, "*")).unwrap_err();
        match err {
            ImportError::InvalidRegex(_) => {}
            _ => {
                panic!("unexpected type of error: {:?}", err);
            }
        }
    }

    #[test]
    fn field_match_captures_creditor_debtor() {
        let entry = test_entry();
        let without_ultimate = xmlnode::TransactionDetails {
            related_parties: Some(xmlnode::RelatedParties {
                debtor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "expected debtor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                creditor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "expected creditor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                creditor_account: None,
                debtor_account: None,
                ultimate_debtor: None,
                ultimate_creditor: None,
            }),
            ..test_transaction()
        };
        let with_ultimate = xmlnode::TransactionDetails {
            related_parties: Some(xmlnode::RelatedParties {
                debtor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "expected debtor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                creditor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "expected creditor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                creditor_account: None,
                debtor_account: None,
                ultimate_debtor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "expected ultimate debtor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
                ultimate_creditor: Some(xmlnode::RelatedParty::from_inner(xmlnode::PartyDetails {
                    name: "expected ultimate creditor".to_string(),
                    ..xmlnode::PartyDetails::default()
                })),
            }),
            ..test_transaction()
        };
        assert_eq!(
            None,
            FieldMatch::try_from((config::RewriteField::CreditorName, "no match"))
                .unwrap()
                .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
        assert_eq!(
            Some(Matched::default()),
            FieldMatch::try_from((config::RewriteField::CreditorName, "expected creditor"))
                .unwrap()
                .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
        assert_eq!(
            Some(Matched {
                payee: Some("expected"),
                code: Some("creditor")
            }),
            FieldMatch::try_from((
                config::RewriteField::CreditorName,
                "(?P<payee>expected) (?P<code>creditor)"
            ))
            .unwrap()
            .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
        assert_eq!(
            None,
            FieldMatch::try_from((config::RewriteField::DebtorName, "no match"))
                .unwrap()
                .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
        assert_eq!(
            Some(Matched::default()),
            FieldMatch::try_from((config::RewriteField::DebtorName, "expected debtor"))
                .unwrap()
                .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
        assert_eq!(
            None,
            FieldMatch::try_from((config::RewriteField::UltimateDebtorName, "something"))
                .unwrap()
                .captures(&Fragment::default(), (&entry, Some(&without_ultimate)))
        );
        assert_eq!(
            None,
            FieldMatch::try_from((config::RewriteField::UltimateDebtorName, "something"))
                .unwrap()
                .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
        assert_eq!(
            Some(Matched::default()),
            FieldMatch::try_from((
                config::RewriteField::UltimateDebtorName,
                "expected ultimate debtor"
            ))
            .unwrap()
            .captures(&Fragment::default(), (&entry, Some(&with_ultimate)))
        );
    }

    #[test]
    fn field_match_remittance_info_no_txn() {
        let entry = test_entry();
        let txn = xmlnode::TransactionDetails {
            remittance_info: Some(xmlnode::RemittanceInfo { unstructured: None }),
            ..test_transaction()
        };
        let m = FieldMatch::try_from((
            config::RewriteField::RemittanceUnstructuredInfo,
            "remittance info",
        ))
        .unwrap();

        assert_eq!(None, m.captures(&Fragment::default(), (&entry, None)));
        assert_eq!(None, m.captures(&Fragment::default(), (&entry, Some(&txn))));
    }

    #[test]
    fn field_match_remittance_info_no_match() {
        let entry = test_entry();
        let txn = xmlnode::TransactionDetails {
            remittance_info: Some(xmlnode::RemittanceInfo {
                unstructured: Some("expected remittance info".to_owned()),
            }),
            ..test_transaction()
        };
        let m =
            FieldMatch::try_from((config::RewriteField::RemittanceUnstructuredInfo, "no match"))
                .unwrap();

        assert_eq!(None, m.captures(&Fragment::default(), (&entry, Some(&txn))));
    }

    #[test]
    fn field_match_remittance_info_match() {
        let entry = test_entry();
        let txn = xmlnode::TransactionDetails {
            remittance_info: Some(xmlnode::RemittanceInfo {
                unstructured: Some("expected remittance info".to_owned()),
            }),
            ..test_transaction()
        };
        let m = FieldMatch::try_from((
            config::RewriteField::RemittanceUnstructuredInfo,
            "expected remittance info",
        ))
        .unwrap();

        assert_eq!(
            Some(Matched::default()),
            m.captures(&Fragment::default(), (&entry, Some(&txn)))
        );
    }

    #[test]
    fn field_match_additional_transaction_info_match() {
        let entry = test_entry();
        let txn = xmlnode::TransactionDetails {
            additional_info: Some("expected additional transaction info".to_owned()),
            ..test_transaction()
        };
        let m = FieldMatch::try_from((
            config::RewriteField::AdditionalTransactionInfo,
            "expected additional transaction info",
        ))
        .unwrap();

        assert_eq!(
            Some(Matched::default()),
            m.captures(&Fragment::default(), (&entry, Some(&txn)))
        );
    }

    #[test]
    fn field_match_payee_match() {
        let fragment = Fragment {
            payee: Some("expected payee"),
            ..Fragment::default()
        };
        let m: FieldMatch = (config::RewriteField::Payee, "expected payee")
            .try_into()
            .unwrap();

        assert_eq!(
            Some(Matched::default()),
            m.captures(&fragment, (&test_entry(), None))
        );
    }
}
