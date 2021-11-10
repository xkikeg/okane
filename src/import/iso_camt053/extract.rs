use super::xmlnode;
use crate::import::config;
use crate::import::error::ImportError;

use std::convert::TryFrom;
use std::convert::TryInto;

use regex::Regex;

#[derive(Debug)]
pub struct Extractor<'a> {
    rules: Vec<ExtractRule<'a>>,
}

pub fn from_config(config: &config::ConfigEntry) -> Result<Extractor, ImportError> {
    from_config_rewrite(&config.rewrite)
}

fn from_config_rewrite<'a>(
    rewrite: &'a [config::RewriteRule],
) -> Result<Extractor<'a>, ImportError> {
    let rules = rewrite
        .iter()
        .map(ExtractRule::try_from)
        .collect::<Result<Vec<ExtractRule<'a>>, ImportError>>()?;
    Ok(Extractor { rules })
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fragment<'a> {
    pub cleared: bool,
    pub payee: Option<&'a str>,
    pub account: Option<&'a str>,
}

impl<'a> std::ops::AddAssign for Fragment<'a> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: Self) {
        self.cleared = other.cleared || self.cleared;
        self.payee = other.payee.or(self.payee);
        self.account = other.account.or(self.account);
    }
}

impl<'a> Extractor<'a> {
    pub fn extract(
        &'a self,
        entry: &'a xmlnode::Entry,
        transaction: Option<&'a xmlnode::TransactionDetails>,
    ) -> Fragment<'a> {
        let mut fragment = Fragment {
            cleared: false,
            payee: None,
            account: None,
        };
        for rule in &self.rules {
            if let Some(updated) = rule.extract(fragment.clone(), entry, transaction) {
                fragment += updated;
            }
        }
        fragment
    }
}

#[derive(Debug)]
struct ExtractRule<'a> {
    match_expr: ExtractMatchOrExpr,
    pending: bool,
    payee: Option<&'a str>,
    account: Option<&'a str>,
}

impl<'a> TryFrom<&'a config::RewriteRule> for ExtractRule<'a> {
    type Error = ImportError;

    fn try_from(config_rule: &'a config::RewriteRule) -> Result<Self, Self::Error> {
        let match_expr = (&config_rule.matcher).try_into()?;
        Ok(ExtractRule {
            match_expr,
            pending: config_rule.pending,
            payee: config_rule.payee.as_deref(),
            account: config_rule.account.as_deref(),
        })
    }
}

impl<'a> ExtractRule<'a> {
    fn extract(
        &self,
        current: Fragment<'a>,
        entry: &'a xmlnode::Entry,
        transaction: Option<&'a xmlnode::TransactionDetails>,
    ) -> Option<Fragment<'a>> {
        self.match_expr
            .extract(current, entry, transaction)
            .map(|mut current| {
                current.payee = self.payee.or(current.payee);
                current.account = self.account;
                if current.payee.is_some() && current.account.is_some() {
                    current.cleared = current.cleared || !self.pending;
                }
                current
            })
    }
}

#[derive(Debug)]
struct ExtractMatchOrExpr(Vec<ExtractMatchAndExpr>);

impl TryFrom<&config::RewriteMatcher> for ExtractMatchOrExpr {
    type Error = ImportError;

    fn try_from(from: &config::RewriteMatcher) -> Result<Self, ImportError> {
        match from {
            config::RewriteMatcher::Or(orms) => {
                let exprs: Result<Vec<ExtractMatchAndExpr>, ImportError> =
                    orms.iter().map(|x| x.try_into()).collect();
                Ok(ExtractMatchOrExpr(exprs?))
            }
            config::RewriteMatcher::Field(f) => {
                let and_expr = f.try_into()?;
                Ok(ExtractMatchOrExpr(vec![and_expr]))
            }
        }
    }
}

impl ExtractMatchOrExpr {
    fn extract<'a>(
        &self,
        current: Fragment<'a>,
        entry: &'a xmlnode::Entry,
        transaction: Option<&'a xmlnode::TransactionDetails>,
    ) -> Option<Fragment<'a>> {
        self.0
            .iter()
            .find_map(|m| m.extract(current.clone(), entry, transaction))
    }
}

#[derive(Debug)]
struct ExtractMatchAndExpr(Vec<ExtractMatch>);

impl TryFrom<&config::FieldMatcher> for ExtractMatchAndExpr {
    type Error = ImportError;

    fn try_from(from: &config::FieldMatcher) -> Result<Self, ImportError> {
        let matchers = from
            .fields
            .iter()
            .map(|(fd, v)| to_extract_match(*fd, v))
            .collect::<Result<Vec<ExtractMatch>, ImportError>>()?;
        if matchers.is_empty() {
            Err(ImportError::InvalidConfig(
                "empty field matcher is not allowed",
            ))
        } else {
            Ok(ExtractMatchAndExpr(matchers))
        }
    }
}

impl ExtractMatchAndExpr {
    fn extract<'a>(
        &self,
        current: Fragment<'a>,
        entry: &'a xmlnode::Entry,
        transaction: Option<&'a xmlnode::TransactionDetails>,
    ) -> Option<Fragment<'a>> {
        self.0.iter().try_fold(current.clone(), |prev, m| {
            m.extract(prev, entry, transaction)
        })
    }
}
#[derive(Debug)]
enum ExtractMatch {
    DomainCode(xmlnode::DomainCode),
    DomainFamily(xmlnode::DomainFamilyCode),
    DomainSubFamily(xmlnode::DomainSubFamilyCode),
    FieldMatch(MatchField, Regex),
}

#[derive(Debug, PartialEq)]
enum MatchField {
    CreditorName,
    UltimateCreditorName,
    DebtorName,
    UltimateDebtorName,
    AdditionalTransactionInfo,
    Payee,
}

fn to_field(f: config::RewriteField) -> Result<MatchField, ImportError> {
    match f {
        config::RewriteField::CreditorName => Some(MatchField::CreditorName),
        config::RewriteField::UltimateCreditorName => Some(MatchField::UltimateCreditorName),
        config::RewriteField::DebtorName => Some(MatchField::DebtorName),
        config::RewriteField::UltimateDebtorName => Some(MatchField::UltimateDebtorName),
        config::RewriteField::AdditionalTransactionInfo => {
            Some(MatchField::AdditionalTransactionInfo)
        }
        config::RewriteField::Payee => Some(MatchField::Payee),
        _ => None,
    }
    .ok_or_else(|| ImportError::Other(format!("unknown match field: {:?}", f)))
}

fn to_extract_match(f: config::RewriteField, v: &str) -> Result<ExtractMatch, ImportError> {
    Ok(match f {
        config::RewriteField::DomainCode => {
            let code = serde_yaml::from_str(v)?;
            ExtractMatch::DomainCode(code)
        }
        config::RewriteField::DomainFamily => {
            let code = serde_yaml::from_str(v)?;
            ExtractMatch::DomainFamily(code)
        }
        config::RewriteField::DomainSubFamily => {
            let code = serde_yaml::from_str(v)?;
            ExtractMatch::DomainSubFamily(code)
        }
        _ => {
            let pattern = Regex::new(v)?;
            let field = to_field(f)?;
            ExtractMatch::FieldMatch(field, pattern)
        }
    })
}

impl ExtractMatch {
    fn extract<'a>(
        &self,
        fragment: Fragment<'a>,
        entry: &'a xmlnode::Entry,
        transaction: Option<&'a xmlnode::TransactionDetails>,
    ) -> Option<Fragment<'a>> {
        let mut fragment = fragment;
        let matched = match self {
            ExtractMatch::DomainCode(code) => {
                *code == entry.bank_transaction_code.domain.code.value
            }
            ExtractMatch::DomainFamily(code) => {
                *code == entry.bank_transaction_code.domain.family.code.value
            }
            ExtractMatch::DomainSubFamily(code) => {
                *code
                    == entry
                        .bank_transaction_code
                        .domain
                        .family
                        .sub_family_code
                        .value
            }
            ExtractMatch::FieldMatch(fd, re) => {
                let target: Option<&str> = match fd {
                    MatchField::CreditorName => {
                        transaction.map(|t| t.related_parties.creditor.name.as_str())
                    }
                    MatchField::UltimateCreditorName => transaction
                        .and_then(|t| t.related_parties.ultimate_creditor.as_ref())
                        .map(|ud| ud.name.as_str()),
                    MatchField::DebtorName => {
                        transaction.map(|t| t.related_parties.debtor.name.as_str())
                    }
                    MatchField::UltimateDebtorName => transaction
                        .and_then(|t| t.related_parties.ultimate_debtor.as_ref())
                        .map(|ud| ud.name.as_str()),
                    MatchField::AdditionalTransactionInfo => {
                        transaction.map(|t| t.additional_info.as_str())
                    }
                    MatchField::Payee => fragment.payee,
                };
                match target.and_then(|t| re.captures(t)) {
                    None => false,
                    Some(c) => {
                        if let Some(v) = c.name("payee") {
                            fragment.payee = Some(v.as_str());
                        }
                        true
                    }
                }
            }
        };
        if matched {
            Some(fragment)
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
                    account_servicer_reference: "foobar".to_string(),
                },
                credit_or_debit: xmlnode::CreditDebitIndicator {
                    value: xmlnode::CreditOrDebit::Credit,
                },
                amount: xmlnode::Amount {
                    value: dec!(12.3),
                    currency: "CHF".to_string(),
                },
                amount_details: xmlnode::AmountDetails {
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
                },
                charges: None,
                related_parties: xmlnode::RelatedParties {
                    debtor: xmlnode::Party {
                        name: "debtor".to_string(),
                    },
                    creditor: xmlnode::Party {
                        name: "creditor".to_string(),
                    },
                    ultimate_debtor: None,
                    ultimate_creditor: None,
                },
                remittance_info: None,
                additional_info: v.additional_info.to_string(),
            }
        }
    }

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

        let extractor = from_config_rewrite(&rw).unwrap();
        let fragment = extractor.extract(&input, None);

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

        let extractor = from_config_rewrite(&rw).unwrap();
        let got: Vec<Fragment> = input
            .details
            .transactions
            .iter()
            .map(|t| extractor.extract(&input, Some(t)))
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
        let result = from_config_rewrite(&rw).unwrap_err();
        match result {
            ImportError::YAML(cause) => {
                assert!(
                    cause.to_string().contains("unknown variant `foo`"),
                    "{:?} did not contains expected error",
                    cause
                );
            }
            _ => {
                panic!("unexpected type of error: {:?}", result);
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
        let result = from_config_rewrite(&rw).unwrap_err();
        match result {
            ImportError::InvalidRegex(_) => {}
            _ => {
                panic!("unexpected type of error: {:?}", result);
            }
        }
    }
}
