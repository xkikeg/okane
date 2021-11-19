mod xmlnode;

use super::config;
use super::extract;
use super::single_entry;
use super::ImportError;
use crate::data;

use std::convert::{TryFrom,TryInto};

use regex::Regex;
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
        let extractor: extract::Extractor<FieldMatch> = (&config.rewrite).try_into()?;
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
                    let fragment = extractor.extract((&entry, None));
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
                    let fragment = extractor.extract((&entry, Some(transaction)));
                    let code = transaction.refs.account_servicer_reference.as_deref();
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
            let payee = config.operator.as_ref().ok_or(ImportError::InvalidConfig(
                "config should have operator to have charge",
            ))?;
            log::info!("ADDED cr: {:?}", cr);
            if !cr.is_charge_included {
                txn.try_add_charge_not_included(
                    payee,
                    cr.amount.to_data(cr.credit_or_debit.value),
                )?;
            } else {
                txn.add_charge(payee, cr.amount.to_data(cr.credit_or_debit.value));
            }
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
    UltimateCreditorName,
    DebtorName,
    UltimateDebtorName,
    RemittanceUnstructuredInfo,
    AdditionalTransactionInfo,
    Payee,
}

fn to_field(f: config::RewriteField) -> Result<MatchField, ImportError> {
    match f {
        config::RewriteField::CreditorName => Some(MatchField::CreditorName),
        config::RewriteField::UltimateCreditorName => Some(MatchField::UltimateCreditorName),
        config::RewriteField::DebtorName => Some(MatchField::DebtorName),
        config::RewriteField::UltimateDebtorName => Some(MatchField::UltimateDebtorName),
        config::RewriteField::RemittanceUnstructuredInfo => {
            Some(MatchField::RemittanceUnstructuredInfo)
        }
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
    fn try_from(from: (config::RewriteField, &str)) -> Result<FieldMatch, ImportError> {
        let f = from.0;
        let v = from.1;
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
                let pattern = Regex::new(v)?;
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
        entity: (&'a xmlnode::Entry, Option<&'a xmlnode::TransactionDetails>),
    ) -> Option<extract::Matched<'a>> {
        let mut matched = extract::Matched::default();
        let entry = &entity.0;
        let transaction = &entity.1;
        let has_match = match self {
            FieldMatch::DomainCode(code) => *code == entry.bank_transaction_code.domain.code.value,
            FieldMatch::DomainFamily(code) => {
                *code == entry.bank_transaction_code.domain.family.code.value
            }
            FieldMatch::DomainSubFamily(code) => {
                *code
                    == entry
                        .bank_transaction_code
                        .domain
                        .family
                        .sub_family_code
                        .value
            }
            FieldMatch::RegexMatch(fd, re) => {
                let target: Option<&str> = match fd {
                    MatchField::CreditorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .map(|rp| rp.creditor.name.as_str()),
                    MatchField::UltimateCreditorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.ultimate_creditor.as_ref())
                        .map(|ud| ud.name.as_str()),
                    MatchField::DebtorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .map(|rp| rp.debtor.name.as_str()),
                    MatchField::UltimateDebtorName => transaction
                        .and_then(|t| t.related_parties.as_ref())
                        .and_then(|rp| rp.ultimate_debtor.as_ref())
                        .map(|ud| ud.name.as_str()),
                    MatchField::RemittanceUnstructuredInfo => transaction
                        .and_then(|t| t.remittance_info.as_ref())
                        .and_then(|i| i.unstructured.as_ref())
                        .map(|v| v.as_str()),
                    MatchField::AdditionalTransactionInfo => transaction
                        .and_then(|t| t.additional_info.as_ref())
                        .map(|ai| ai.as_str()),
                    MatchField::Payee => fragment.payee,
                };
                match target.and_then(|t| re.captures(t)) {
                    None => false,
                    Some(c) => {
                        if let Some(v) = c.name("payee") {
                            matched.payee = Some(v.as_str());
                        }
                        true
                    }
                }
            }
        };
        if has_match {
            Some(matched)
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fragment_add_assign_filled() {
        let mut x = Fragment {
            cleared: true,
            payee: Some("foo"),
            account: None,
        };
        let y = Fragment {
            cleared: false,
            payee: Some("bar"),
            account: Some("baz"),
        };
        x += y;
        assert_eq!(
            x,
            Fragment {
                cleared: true,
                payee: Some("bar"),
                account: Some("baz")
            }
        );
    }

    #[test]
    fn test_fragment_add_assign_empty() {
        let mut x = Fragment {
            cleared: false,
            payee: Some("foo"),
            account: None,
        };
        let y = Fragment {
            cleared: false,
            payee: None,
            account: None,
        };
        x += y;
        assert_eq!(
            x,
            Fragment {
                cleared: false,
                payee: Some("foo"),
                account: None
            }
        );
    }

    struct Ntry {
        domain_code: xmlnode::DomainCode,
        domain_family: xmlnode::DomainFamilyCode,
        domain_sub_family: xmlnode::DomainSubFamilyCode,
        txns: Vec<Txn>,
    }

    struct Txn {
        additional_info: &'static str,
    }

    impl From<Ntry> for xmlnode::Entry {
        fn from(v: Ntry) -> Self {
            xmlnode::Entry {
                amount: xmlnode::Amount {
                    value: dec!(120),
                    currency: "CHF".to_string(),
                },
                credit_or_debit: xmlnode::CreditDebitIndicator {
                    value: xmlnode::CreditOrDebit::Credit,
                },
                booking_date: xmlnode::Date {
                    date: NaiveDate::from_ymd(2021, 10, 1),
                },
                value_date: xmlnode::Date {
                    date: NaiveDate::from_ymd(2021, 10, 1),
                },
                bank_transaction_code: xmlnode::BankTransactionCode {
                    domain: xmlnode::Domain {
                        code: xmlnode::DomainCodeValue {
                            value: v.domain_code,
                        },
                        family: xmlnode::DomainFamily {
                            code: xmlnode::DomainFamilyCodeValue {
                                value: v.domain_family,
                            },
                            sub_family_code: xmlnode::DomainSubFamilyCodeValue {
                                value: v.domain_sub_family,
                            },
                        },
                    },
                },
                charges: None,
                additional_info: "entry additional info".to_string(),
                details: xmlnode::EntryDetails {
                    batch: xmlnode::Batch {
                        number_of_transactions: v.txns.len(),
                    },
                    transactions: v.txns.into_iter().map(Into::into).collect(),
                },
            }
        }
    }

    impl From<Txn> for xmlnode::TransactionDetails {
        fn from(v: Txn) -> xmlnode::TransactionDetails {
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
                    instructed: xmlnode::AmountWithExchange {
                        amount: xmlnode::Amount {
                            value: dec!(12.3),
                            currency: "CHF".to_string(),
                        },
                        currency_exchange: None,
                    },
                    transaction: xmlnode::AmountWithExchange {
                        amount: xmlnode::Amount {
                            value: dec!(12.3),
                            currency: "CHF".to_string(),
                        },
                        currency_exchange: None,
                    },
                }),
                charges: None,
                related_parties: Some(xmlnode::RelatedParties {
                    debtor: xmlnode::Party {
                        name: "debtor".to_string(),
                    },
                    creditor: xmlnode::Party {
                        name: "creditor".to_string(),
                    },
                    ultimate_debtor: None,
                    ultimate_creditor: None,
                }),
                remittance_info: None,
                additional_info: Some(v.additional_info.to_string()),
            }
        }
    }

    use extract::Fragment;

    #[test]
    fn test_from_config_single_match() {
        let rw = vec![config::RewriteRule {
            matcher: config::RewriteMatcher::Field(config::FieldMatcher {
                fields: hashmap! {
                    config::RewriteField::DomainCode => "PMNT".to_string(),
                    config::RewriteField::DomainFamily => "RCDT".to_string(),
                    config::RewriteField::DomainSubFamily => "SALA".to_string(),
                },
            }),
            pending: false,
            payee: Some("Payee".to_string()),
            account: Some("Income".to_string()),
        }];
        let input = Ntry {
            domain_code: xmlnode::DomainCode::Payment,
            domain_family: xmlnode::DomainFamilyCode::ReceivedCreditTransfers,
            domain_sub_family: xmlnode::DomainSubFamilyCode::Salary,
            txns: Vec::new(),
        }
        .into();
        let want = Fragment {
            cleared: true,
            account: Some("Income"),
            payee: Some("Payee"),
        };

        let extractor: extract::Extractor<FieldMatch> = (&rw).try_into().unwrap();
        let fragment = extractor.extract((&input, None));

        assert_eq!(want, fragment);
    }

    #[test]
    fn test_from_config_multi_match() {
        let rw = vec![
            config::RewriteRule {
                matcher: config::RewriteMatcher::Field(config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::AdditionalTransactionInfo => r#"Some card (?P<payee>.*)"#.to_string(),
                    },
                }),
                pending: false, // pending: true implied
                payee: None,
                account: None,
            },
            config::RewriteRule {
                matcher: config::RewriteMatcher::Or(vec![
                    config::FieldMatcher {
                        fields: hashmap! {
                            config::RewriteField::Payee => "Grocery shop".to_string(),
                        },
                    },
                    config::FieldMatcher {
                        fields: hashmap! {
                            config::RewriteField::Payee => "Another shop".to_string(),
                        },
                    },
                ]),
                pending: false,
                payee: None,
                account: Some("Expenses:Grocery".to_string()),
            },
            config::RewriteRule {
                matcher: config::RewriteMatcher::Field(config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::Payee => "Certain Petrol".to_string(),
                    },
                }),
                pending: true,
                payee: None,
                account: Some("Expenses:Petrol".to_string()),
            },
        ];
        let input: xmlnode::Entry = Ntry {
            domain_code: xmlnode::DomainCode::Payment,
            domain_family: xmlnode::DomainFamilyCode::ReceivedDirectDebits,
            domain_sub_family: xmlnode::DomainSubFamilyCode::Other,
            txns: vec![
                Txn {
                    additional_info: "Some card Grocery shop",
                },
                Txn {
                    additional_info: "Some card Another shop",
                },
                Txn {
                    additional_info: "Some card Certain Petrol",
                },
                Txn {
                    additional_info: "Some card unknown payee",
                },
                Txn {
                    additional_info: "unrelated",
                },
            ],
        }
        .into();
        let want = vec![
            Fragment {
                cleared: true,
                account: Some("Expenses:Grocery"),
                payee: Some("Grocery shop"),
            },
            Fragment {
                cleared: true,
                account: Some("Expenses:Grocery"),
                payee: Some("Another shop"),
            },
            Fragment {
                cleared: false,
                account: Some("Expenses:Petrol"),
                payee: Some("Certain Petrol"),
            },
            Fragment {
                cleared: false,
                account: None,
                payee: Some("unknown payee"),
            },
            Fragment {
                cleared: false,
                account: None,
                payee: None,
            },
        ];

        let extractor: extract::Extractor<FieldMatch> = (&rw).try_into().unwrap();
        let got: Vec<Fragment> = input
            .details
            .transactions
            .iter()
            .map(|t| extractor.extract((&input, Some(t))))
            .collect();
        assert_eq!(want, got);
    }

    #[test]
    fn test_from_config_invalid_domain_code() {
        let rw = vec![config::RewriteRule {
            matcher: config::RewriteMatcher::Field(config::FieldMatcher {
                fields: hashmap! {
                    config::RewriteField::DomainCode => "foo".to_string(),
                },
            }),
            pending: false,
            payee: None,
            account: None,
        }];
        let result: Result<extract::Extractor<FieldMatch>, ImportError> = (&rw).try_into();
        let err = result.unwrap_err();
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
    fn test_from_config_invalid_regex() {
        let rw = vec![config::RewriteRule {
            matcher: config::RewriteMatcher::Field(config::FieldMatcher {
                fields: hashmap! {
                    config::RewriteField::AdditionalTransactionInfo => "*".to_string(),
                },
            }),
            pending: false,
            payee: None,
            account: None,
        }];
        let result: Result<extract::Extractor<FieldMatch>, ImportError> = (&rw).try_into();
        let err = result.unwrap_err();
        match err {
            ImportError::InvalidRegex(_) => {}
            _ => {
                panic!("unexpected type of error: {:?}", err);
            }
        }
    }
}
