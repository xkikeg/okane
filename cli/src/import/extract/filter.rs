//! Defines [`FieldFilter`].

use either::Either;
use regex::Regex;

use crate::import::error::{ImportError, ImportErrorKind, IntoImportError};
use crate::import::{config, iso_camt053::xmlnode};

use super::CamtStrField;
use super::Entity;
use super::EntityFormat;
use super::Fragment;
use super::StrField;

/// Filter matching one particular field condition.
/// See [`super::extract::Extractor`].
#[derive(Debug)]
pub(super) enum FieldFilter {
    DomainCode(xmlnode::DomainCode),
    DomainFamily(xmlnode::DomainFamilyCode),
    DomainSubFamily(xmlnode::DomainSubFamilyCode),
    RegexMatch(StrField, Regex),
}

impl FieldFilter {
    /// Creates a new instance.
    pub(super) fn try_new<Ef: EntityFormat>(
        f: config::RewriteField,
        v: &str,
        entity_format: Ef,
    ) -> Result<Self, ImportError> {
        let check_camt = || {
            // Fragment doesn't have Camt specific codes, so only check the format.
            if !entity_format.has_camt_transaction_code() {
                Err(ImportError::new(
                    ImportErrorKind::InvalidConfig,
                    format!(
                        "Format {} doesn't support match against {f}",
                        entity_format.name()
                    ),
                ))
            } else {
                Ok(())
            }
        };
        let field = match f {
            config::RewriteField::DomainCode => {
                check_camt()?;
                let code = serde_yaml::from_str(v)
                    .into_import_err(ImportErrorKind::InvalidConfig, || {
                        format!("matcher has invalid domain code {v}")
                    })?;
                return Ok(Self::DomainCode(code));
            }
            config::RewriteField::DomainFamily => {
                check_camt()?;
                let code = serde_yaml::from_str(v)
                    .into_import_err(ImportErrorKind::InvalidConfig, || {
                        format!("matcher has invalid domain family {v}")
                    })?;
                return Ok(Self::DomainFamily(code));
            }
            config::RewriteField::DomainSubFamily => {
                check_camt()?;
                let code = serde_yaml::from_str(v)
                    .into_import_err(ImportErrorKind::InvalidConfig, || {
                        format!("matcher has invalid domain sub family {v}")
                    })?;
                return Ok(Self::DomainSubFamily(code));
            }
            config::RewriteField::Payee => StrField::Payee,
            config::RewriteField::Category => StrField::Category,
            config::RewriteField::Commodity => StrField::Commodity,
            config::RewriteField::SecondaryCommodity => StrField::SecondaryCommodity,
            config::RewriteField::CreditorName => StrField::Camt(CamtStrField::CreditorName),
            config::RewriteField::CreditorAccountId => {
                StrField::Camt(CamtStrField::CreditorAccountId)
            }
            config::RewriteField::UltimateCreditorName => {
                StrField::Camt(CamtStrField::UltimateCreditorName)
            }
            config::RewriteField::DebtorName => StrField::Camt(CamtStrField::DebtorName),
            config::RewriteField::DebtorAccountId => StrField::Camt(CamtStrField::DebtorAccountId),
            config::RewriteField::UltimateDebtorName => {
                StrField::Camt(CamtStrField::UltimateDebtorName)
            }
            config::RewriteField::RemittanceUnstructuredInfo => {
                StrField::Camt(CamtStrField::RemittanceUnstructuredInfo)
            }
            config::RewriteField::AdditionalEntryInfo => {
                StrField::Camt(CamtStrField::AdditionalEntryInfo)
            }
            config::RewriteField::AdditionalTransactionInfo => {
                StrField::Camt(CamtStrField::AdditionalTransactionInfo)
            }
        };
        // Since Fragment always exists, Fragment supported fields are always supported.
        if !Fragment::has_str_field(field) && !entity_format.has_str_field(field) {
            return Err(ImportError::new(
                ImportErrorKind::InvalidConfig,
                format!(
                    "Format {} doesn't support match against {f}",
                    entity_format.name()
                ),
            ));
        }
        let pattern = regex_matcher(v, field)?;
        Ok(Self::RegexMatch(field, pattern))
    }

    /// Checks if the filter matches the given fragment and entity,
    /// and returns matched if captured item is found.
    pub(super) fn captures<'a, Et: Entity<'a>>(
        &self,
        fragment: &Fragment<'a>,
        entity: Et,
    ) -> Option<Matched<'a>> {
        let has_match = match self {
            Self::DomainCode(code) => {
                Either::Left(*code == entity.camt_transaction_code()?.domain_code()?)
            }
            Self::DomainFamily(code) => {
                Either::Left(*code == entity.camt_transaction_code()?.domain_family_code()?)
            }
            Self::DomainSubFamily(code) => {
                Either::Left(*code == entity.camt_transaction_code()?.domain_sub_family_code()?)
            }
            Self::RegexMatch(fd, re) => {
                let target: Option<&str> =
                    fragment.str_field(*fd).or_else(|| entity.str_field(*fd));
                log::trace!("trying to match field {fd:?} regex {re:?} against target {target:?}");
                Either::Right(target.and_then(|t| re.captures(t)).map(Matched::from))
            }
        };
        has_match.right_or_else(|matched| {
            if matched {
                Some(Matched::default())
            } else {
                None
            }
        })
    }
}

/// Creates standard regex matcher.
fn regex_matcher(value: &str, field: StrField) -> Result<regex::Regex, ImportError> {
    regex::RegexBuilder::new(value)
        .case_insensitive(true)
        .build()
        .into_import_err(ImportErrorKind::InvalidConfig, || {
            format!("field {field} matcher value {value} is invalid regex")
        })
}

/// Matched is a result of [`FieldFilter::captures`] method,
/// which would feed regex captured data into [`Fragment`].
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Matched<'a> {
    pub payee: Option<&'a str>,
    pub code: Option<&'a str>,
}

impl<'a> From<regex::Captures<'a>> for Matched<'a> {
    fn from(from: regex::Captures<'a>) -> Self {
        Matched {
            payee: from.name("payee").map(|x| x.as_str()),
            code: from.name("code").map(|x| x.as_str()),
        }
    }
}

impl<'a> std::ops::Add<Matched<'a>> for Fragment<'a> {
    type Output = Self;
    fn add(self, rhs: Matched<'a>) -> Self::Output {
        Fragment {
            payee: rhs.payee.or(self.payee),
            code: rhs.code.or(self.code),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{HashMap, HashSet};
    use std::error::Error;

    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};

    use crate::import::extract::testing::{TestEntity, TestFormat};

    #[fixture]
    fn camt_format() -> TestFormat {
        TestFormat {
            has_all: false,
            has_camt: true,
            has_str: HashSet::new(),
        }
    }

    #[rstest]
    #[case(config::RewriteField::DomainCode)]
    #[case(config::RewriteField::DomainFamily)]
    #[case(config::RewriteField::DomainSubFamily)]
    fn invalid_camt_domains(camt_format: TestFormat, #[case] field: config::RewriteField) {
        let got_err = FieldFilter::try_new(field, "foo", &camt_format).unwrap_err();

        assert_eq!(ImportErrorKind::InvalidConfig, got_err.error_kind());
        let cause = got_err.source().expect("this failure should have cause");
        assert!(
            cause.to_string().contains("unknown variant `foo`"),
            "{:?} did not contains expected error",
            cause
        );
    }

    #[rstest]
    #[case(config::RewriteField::DomainCode)]
    #[case(config::RewriteField::DomainFamily)]
    #[case(config::RewriteField::DomainSubFamily)]
    fn unsupported_camt(#[case] field: config::RewriteField) {
        let got_err = FieldFilter::try_new(field, "foo", &TestFormat::none()).unwrap_err();

        assert_eq!(ImportErrorKind::InvalidConfig, got_err.error_kind());
        assert!(
            got_err
                .message()
                .contains("doesn't support match against domain")
        );
    }

    #[rstest]
    #[case(config::RewriteField::DomainCode, "PMNT")]
    #[case(config::RewriteField::DomainFamily, "ICDT")]
    #[case(config::RewriteField::DomainSubFamily, "AUTT")]
    fn camt_domain_matches(
        camt_format: TestFormat,
        #[case] field: config::RewriteField,
        #[case] cond: &str,
    ) {
        let filter = FieldFilter::try_new(field, cond, &camt_format).unwrap();
        let fragment = Fragment::default();
        let entity = TestEntity {
            camt_txn_code: Some(xmlnode::BankTransactionCode {
                domain: Some(xmlnode::Domain {
                    code: xmlnode::DomainCodeValue {
                        // PMNT
                        value: xmlnode::DomainCode::Payment,
                    },
                    family: xmlnode::DomainFamily {
                        code: xmlnode::DomainFamilyCodeValue {
                            // ICDT
                            value: xmlnode::DomainFamilyCode::IssuedCreditTransfers,
                        },
                        sub_family_code: xmlnode::DomainSubFamilyCodeValue {
                            // AUTT
                            value: xmlnode::DomainSubFamilyCode::AutomaticTransfer,
                        },
                    },
                }),
                proprietary: None,
            }),
            str_fields: HashMap::new(),
        };

        let matched = filter.captures(&fragment, &entity);

        assert_eq!(Some(Matched::default()), matched);
    }

    #[rstest]
    #[case(config::RewriteField::DomainFamily, "ICDT")]
    #[case(config::RewriteField::DomainSubFamily, "AUTT")]
    fn camt_domain_does_not_match(
        camt_format: TestFormat,
        #[case] field: config::RewriteField,
        #[case] cond: &str,
    ) {
        let filter = FieldFilter::try_new(field, cond, &camt_format).unwrap();
        let fragment = Fragment::default();
        let entity = TestEntity {
            camt_txn_code: Some(xmlnode::BankTransactionCode {
                domain: Some(xmlnode::Domain {
                    code: xmlnode::DomainCodeValue {
                        // PMNT
                        value: xmlnode::DomainCode::Payment,
                    },
                    family: xmlnode::DomainFamily {
                        code: xmlnode::DomainFamilyCodeValue {
                            // RCDT
                            value: xmlnode::DomainFamilyCode::ReceivedCreditTransfers,
                        },
                        sub_family_code: xmlnode::DomainSubFamilyCodeValue {
                            // SALA
                            value: xmlnode::DomainSubFamilyCode::Salary,
                        },
                    },
                }),
                proprietary: None,
            }),
            str_fields: HashMap::new(),
        };

        let matched = filter.captures(&fragment, &entity);

        assert_eq!(None, matched);
    }

    #[test]
    fn unsupported_str_field() {
        let got_err = FieldFilter::try_new(config::RewriteField::Category, "", &TestFormat::none())
            .unwrap_err();

        assert_eq!(ImportErrorKind::InvalidConfig, got_err.error_kind());
        assert!(
            got_err
                .message()
                .contains("doesn't support match against category")
        );
    }

    #[test]
    fn invalid_regex_match() {
        let got_err =
            FieldFilter::try_new(config::RewriteField::Payee, "*", &TestFormat::all()).unwrap_err();

        assert_eq!(ImportErrorKind::InvalidConfig, got_err.error_kind());
        assert!(got_err.message().contains("invalid regex"));
    }

    #[test]
    fn payee_works_for_none_format() {
        let got = FieldFilter::try_new(
            config::RewriteField::Payee,
            "this.*pen",
            &TestFormat::none(),
        )
        .unwrap();

        let fragment = Fragment::default();
        let entity = TestEntity::default();

        assert_eq!(None, got.captures(&fragment, &entity));

        let fragment = Fragment {
            payee: Some("this is an apple"),
            ..Fragment::default()
        };
        assert_eq!(None, got.captures(&fragment, &entity));

        let fragment = Fragment {
            payee: Some("this is a pen"),
            ..Fragment::default()
        };
        assert_eq!(Some(Matched::default()), got.captures(&fragment, &entity));
    }
}
