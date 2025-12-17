mod filter;
#[cfg(test)]
mod testing;

use chrono::NaiveDate;

use okane_core::syntax::ClearState;

use super::amount::OwnedAmount;
use super::config;
use super::error::ImportError;
use super::iso_camt053::xmlnode;
use super::single_entry;

use filter::FieldFilter;

/// Extractor is a set of [`ExtractRule`], so to extract [`Fragment`] out of the [`Entity`].
#[derive(Debug)]
pub struct Extractor<'a> {
    rules: Vec<ExtractRule<'a>>,
}

impl<'a> Extractor<'a> {
    /// Create an instance from [`config::ConfigEntry`].
    pub fn from_config<Ef: EntityFormat>(
        entity_format: Ef,
        config: &'a config::ConfigEntry,
    ) -> Result<Self, ImportError> {
        Self::try_new(entity_format, &config.rewrite)
    }

    /// Create an instance from [`config::RewriteRule`] items.
    pub fn try_new<Ef: EntityFormat>(
        entity_format: Ef,
        rules: &'a [config::RewriteRule],
    ) -> Result<Self, ImportError> {
        rules
            .iter()
            .map(|x| ExtractRule::try_new(x, entity_format))
            .collect::<Result<Vec<_>, _>>()
            .map(|rules| Extractor { rules })
    }

    /// Extracts the entity information into [`Fragment`].
    pub fn extract<Et: Entity<'a>>(&'a self, entity: Et) -> Fragment<'a> {
        let mut fragment = Fragment::default();
        for rule in &self.rules {
            if let Some(updated) = rule.extract(fragment.clone(), entity) {
                fragment += updated;
            }
        }
        fragment
    }
}

/// Format definition describing which fields are accepted.
pub trait EntityFormat: Copy {
    /// Name of the format.
    fn name(&self) -> &'static str;

    /// Returns if the format accepts Camt domains.
    fn has_camt_transaction_code(&self) -> bool;

    /// Returns if the format accepts specified field.
    fn has_str_field(&self, field: StrField) -> bool;
}

/// Entity to extract [`Fragment`] with [`Extractor`].
///
/// Usually entity corresponds to one transaction such as a particular CSV row.
pub trait Entity<'a>: Copy {
    /// Returns Camt053 domain code if available.
    fn camt_transaction_code(&self) -> Option<&'a xmlnode::BankTransactionCode> {
        None
    }

    /// Returns `&str` field corresponding to the specified field.
    fn str_field(&self, field: StrField) -> Option<&'a str>;
}

/// String fields.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StrField {
    Payee,
    Category,
    SecondaryCommodity,
    /// Camt specific field.
    /// With this approach other format (CSV) can avoid skipping all camt arm one by one.
    Camt(CamtStrField),
}

/// Camt specific string fields.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CamtStrField {
    CreditorName,
    CreditorAccountId,
    UltimateCreditorName,
    DebtorName,
    DebtorAccountId,
    UltimateDebtorName,
    RemittanceUnstructuredInfo,
    AdditionalEntryInfo,
    AdditionalTransactionInfo,
}

/// Fragment is a extracted information out of parcitular entity.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Fragment<'a> {
    /// True if the entity is clearly classified.
    pub cleared: bool,
    /// Payee of the transaction, `None` if not found.
    pub payee: Option<&'a str>,
    /// Account of the transaction target, `None` if not found.
    pub account: Option<&'a str>,
    /// Code identifying the transaction, `None` if not found.
    pub code: Option<&'a str>,
    /// Currency conversion, `None` if not found.
    pub conversion: Option<&'a config::CommodityConversionSpec>,
}

impl<'a> Fragment<'a> {
    /// Creates a new transaction out of the fragment.
    pub fn new_txn<Pos, PosFn>(
        &self,
        date: NaiveDate,
        amount: OwnedAmount,
        mut position: PosFn,
    ) -> single_entry::Txn
    where
        PosFn: FnMut() -> Pos,
        Pos: std::fmt::Display,
    {
        let payee = match self.payee {
            Some(p) => p,
            None => {
                log::warn!("payee not set @ {}", position());
                "unknown payee"
            }
        };
        let mut txn = single_entry::Txn::new(date, payee, amount);
        txn.code_option(self.code.map(|x| x.into()).into())
            .dest_account_option(self.account);
        if !self.cleared {
            txn.clear_state(ClearState::Pending);
        }
        txn
    }

    /// Returns field existence. Similar to [`EntityFormat::has_str_field`].
    /// For now, fragment only supports payee.
    fn has_str_field(field: StrField) -> bool {
        return field == StrField::Payee;
    }

    /// Returns the corresponding field, similar to [`Entity::str_field`].
    /// For now, fragment only supports payee.
    fn str_field(&self, field: StrField) -> Option<&'a str> {
        match field {
            StrField::Payee => self.payee,
            _ => None,
        }
    }
}

impl std::ops::AddAssign for Fragment<'_> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: Self) {
        *self = Fragment {
            cleared: other.cleared || self.cleared,
            payee: other.payee.or(self.payee),
            account: other.account.or(self.account),
            code: other.code.or(self.code),
            conversion: other.conversion.or(self.conversion),
        };
    }
}

#[derive(Debug)]
struct ExtractRule<'a> {
    match_expr: MatchOrExpr,
    pending: bool,
    payee: Option<&'a str>,
    account: Option<&'a str>,
    conversion: Option<&'a config::CommodityConversionSpec>,
}

impl<'a> ExtractRule<'a> {
    /// Constructs a rule instance out of a RewriteRule.
    fn try_new<Ef: EntityFormat>(
        from: &'a config::RewriteRule,
        entity_format: Ef,
    ) -> Result<Self, ImportError> {
        let match_expr = MatchOrExpr::try_new(&from.matcher, entity_format)?;
        Ok(ExtractRule {
            match_expr,
            pending: from.pending,
            payee: from.payee.as_deref(),
            account: from.account.as_deref(),
            conversion: from.conversion.as_ref(),
        })
    }
}

impl<'a> ExtractRule<'a> {
    fn extract<Et: Entity<'a>>(&self, current: Fragment<'a>, entity: Et) -> Option<Fragment<'a>> {
        self.match_expr.extract(current, entity).map(|mut current| {
            current.payee = self.payee.or(current.payee);
            current.account = self.account;
            current.conversion = self.conversion.or(current.conversion);
            if current.account.is_some() {
                current.cleared = current.cleared || !self.pending;
            }
            current
        })
    }
}

#[derive(Debug)]
struct MatchOrExpr(Vec<MatchAndExpr>);

impl MatchOrExpr {
    /// Creates a new instance.
    fn try_new<Ef: EntityFormat>(
        from: &config::RewriteMatcher,
        entity_format: Ef,
    ) -> Result<Self, ImportError> {
        match from {
            config::RewriteMatcher::Or(orms) => {
                let exprs: Result<Vec<MatchAndExpr>, ImportError> = orms
                    .iter()
                    .map(|x| MatchAndExpr::try_new(x, entity_format))
                    .collect();
                Ok(MatchOrExpr(exprs?))
            }
            config::RewriteMatcher::Field(f) => {
                let and_expr = MatchAndExpr::try_new(f, entity_format)?;
                Ok(MatchOrExpr(vec![and_expr]))
            }
        }
    }

    /// Extracts [`Fragment`].
    fn extract<'a, Et: Entity<'a>>(
        &self,
        current: Fragment<'a>,
        entity: Et,
    ) -> Option<Fragment<'a>> {
        self.0
            .iter()
            .find_map(|m| m.extract(current.clone(), entity))
    }
}

#[derive(Debug)]
struct MatchAndExpr(Vec<FieldFilter>);

impl MatchAndExpr {
    fn try_new<Ef: EntityFormat>(
        from: &config::FieldMatcher,
        entity_format: Ef,
    ) -> Result<Self, ImportError> {
        let matchers: Result<Vec<FieldFilter>, _> = from
            .fields
            .iter()
            .map(|(fd, v)| FieldFilter::try_new(*fd, v.as_str(), entity_format))
            .collect();
        let matchers = matchers?;
        if matchers.is_empty() {
            Err(ImportError::InvalidConfig(
                "empty field matcher is not allowed",
            ))
        } else {
            Ok(MatchAndExpr(matchers))
        }
    }

    fn extract<'a, Et: Entity<'a>>(
        &self,
        current: Fragment<'a>,
        entity: Et,
    ) -> Option<Fragment<'a>> {
        let got = self.0.iter().try_fold(current, |prev, matcher| {
            matcher
                .captures(&prev, entity)
                .map(|matched| prev + matched)
        });
        got
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use config::CommodityConversionSpec;

    use testing::{TestEntity, TestFormat};

    #[test]
    fn fragment_add_assign_filled() {
        let mut x = Fragment {
            cleared: true,
            payee: Some("foo"),
            account: None,
            code: None,
            conversion: None,
        };
        let conversion = CommodityConversionSpec::default();
        let y = Fragment {
            cleared: false,
            payee: Some("bar"),
            account: Some("baz"),
            code: Some("txn-id"),
            conversion: Some(&conversion),
        };
        x += y;
        assert_eq!(
            x,
            Fragment {
                cleared: true,
                payee: Some("bar"),
                account: Some("baz"),
                code: Some("txn-id"),
                conversion: Some(&CommodityConversionSpec::default()),
            }
        );
    }

    #[test]
    fn fragment_add_assign_empty() {
        let conversion = CommodityConversionSpec::default();
        let orig = Fragment {
            cleared: false,
            payee: Some("foo"),
            account: None,
            code: Some("txn-id"),
            conversion: Some(&conversion),
        };
        let mut x = orig.clone();
        x += Fragment::default();
        assert_eq!(x, orig);
    }

    fn into_rule(m: config::RewriteMatcher) -> config::RewriteRule {
        config::RewriteRule {
            matcher: m,
            pending: false,
            payee: None,
            account: None,
            conversion: None,
        }
    }

    #[test]
    fn extract_single_match_all() {
        use strum::EnumCount;

        let fields = hashmap! {
            config::RewriteField::DomainCode => "PMNT".to_string(),
            config::RewriteField::DomainFamily => "RDDT".to_string(),
            config::RewriteField::DomainSubFamily => "STDO".to_string(),
            config::RewriteField::CreditorName => "my-creditor-name".to_string(),
            config::RewriteField::CreditorAccountId => "my-creditor-account-id".to_string(),
            config::RewriteField::UltimateCreditorName => "my-ultimate-creditor-name".to_string(),
            config::RewriteField::DebtorName => "my-debtor-name".to_string(),
            config::RewriteField::DebtorAccountId => "my-debtor-account-id".to_string(),
            config::RewriteField::UltimateDebtorName => "my-ultimate-debtor-name".to_string(),
            config::RewriteField::RemittanceUnstructuredInfo => "my remittance info".to_string(),
            config::RewriteField::AdditionalEntryInfo => "my entry info".to_string(),
            config::RewriteField::AdditionalTransactionInfo => "my transaction info: (?P<payee>.*)".to_string(),
            config::RewriteField::SecondaryCommodity => "CHF".to_string(),
            config::RewriteField::Category => "my category".to_string(),
            config::RewriteField::Payee => "my payee".to_string(),
        };
        assert_eq!(fields.len(), config::RewriteField::COUNT);
        let rw = vec![config::RewriteRule {
            pending: false,
            payee: None,
            account: Some("Income".to_string()),
            ..into_rule(config::RewriteMatcher::Field(config::FieldMatcher {
                fields,
            }))
        }];
        let input = TestEntity {
            camt_txn_code: Some(xmlnode::BankTransactionCode {
                domain: Some(xmlnode::Domain {
                    code: xmlnode::DomainCodeValue {
                        // PMNT
                        value: xmlnode::DomainCode::Payment,
                    },
                    family: xmlnode::DomainFamily {
                        code: xmlnode::DomainFamilyCodeValue {
                            // RDDT
                            value: xmlnode::DomainFamilyCode::ReceivedDirectDebits,
                        },
                        sub_family_code: xmlnode::DomainSubFamilyCodeValue {
                            // STDO
                            value: xmlnode::DomainSubFamilyCode::StandingOrder,
                        },
                    },
                }),
                proprietary: None,
            }),
            str_fields: hashmap! {
                StrField::Camt(CamtStrField::CreditorName) => "my-creditor-name".to_string(),
                StrField::Camt(CamtStrField::CreditorAccountId) => "my-creditor-account-id".to_string(),
                StrField::Camt(CamtStrField::UltimateCreditorName) => "my-ultimate-creditor-name".to_string(),
                StrField::Camt(CamtStrField::DebtorName) => "my-debtor-name".to_string(),
                StrField::Camt(CamtStrField::DebtorAccountId) => "my-debtor-account-id".to_string(),
                StrField::Camt(CamtStrField::UltimateDebtorName) => "my-ultimate-debtor-name".to_string(),
                StrField::Camt(CamtStrField::RemittanceUnstructuredInfo) => "my remittance info".to_string(),
                StrField::Camt(CamtStrField::AdditionalEntryInfo) => "my entry info".to_string(),
                // this must be compatible with Payee matcher.
                // in practical, it's not recommended to use both
                // payee in regex capture and payee filter at the same time.
                StrField::Camt(CamtStrField::AdditionalTransactionInfo) => "my transaction info: my payee new".to_string(),
                StrField::SecondaryCommodity => "CHF".to_string(),
                StrField::Category => "my category".to_string(),
                StrField::Payee => "my payee".to_string(),
            },
        };
        let want = Fragment {
            cleared: true,
            account: Some("Income"),
            payee: Some("my payee new"),
            ..Fragment::default()
        };

        let extractor = Extractor::try_new(&TestFormat::all(), &rw).unwrap();
        let fragment = extractor.extract(&input);

        assert_eq!(want, fragment);
    }

    #[test]
    fn extract_single_match_or() {
        let rw = vec![config::RewriteRule {
            pending: false,
            payee: Some("Payee".to_string()),
            account: Some("Expense".to_string()),
            ..into_rule(config::RewriteMatcher::Or(vec![
                config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::Category => "food".to_string(),
                    },
                },
                config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::Category => "beverage".to_string(),
                    },
                },
            ]))
        }];
        let input = TestEntity {
            camt_txn_code: None,
            str_fields: hashmap! {
                StrField::Category => "beverage".to_string(),
            },
        };
        let want = Fragment {
            cleared: true,
            account: Some("Expense"),
            payee: Some("Payee"),
            ..Fragment::default()
        };

        let extractor = Extractor::try_new(&TestFormat::all(), &rw).unwrap();
        let fragment = extractor.extract(&input);

        assert_eq!(want, fragment);
    }

    #[test]
    fn extract_multi_match() {
        let conversion = config::CommodityConversionSpec {
            commodity: Some("JPY".to_string()),
            ..config::CommodityConversionSpec::default()
        };
        let rw = vec![
            config::RewriteRule {
                pending: false,
                payee: None,
                account: None,
                ..into_rule(config::RewriteMatcher::Field(config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::AdditionalTransactionInfo => r#"Some card(?: \[(?P<code>\d+)\])? (?P<payee>.*)"#.to_string(),
                    },
                }))
            },
            config::RewriteRule {
                pending: false,
                payee: None,
                account: Some("Expenses:Grocery".to_string()),
                ..into_rule(config::RewriteMatcher::Or(vec![
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
                ]))
            },
            config::RewriteRule {
                pending: true,
                payee: None,
                account: Some("Expenses:Petrol".to_string()),
                conversion: Some(conversion.clone()),
                ..into_rule(config::RewriteMatcher::Field(config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::Payee => "Certain Petrol".to_string(),
                    },
                }))
            },
        ];
        let input = [
            TestEntity {
                camt_txn_code: None,
                str_fields: hashmap! {
                    StrField::Camt(CamtStrField::AdditionalTransactionInfo) => "Some card Grocery shop".to_string(),
                },
            },
            TestEntity {
                camt_txn_code: None,
                str_fields: hashmap! {
                    StrField::Camt(CamtStrField::AdditionalTransactionInfo) => "Some card Another shop".to_string(),
                },
            },
            TestEntity {
                camt_txn_code: None,
                str_fields: hashmap! {
                    StrField::Camt(CamtStrField::AdditionalTransactionInfo) => "Some card [123] Certain Petrol".to_string(),
                },
            },
            TestEntity {
                camt_txn_code: None,
                str_fields: hashmap! {
                    StrField::Camt(CamtStrField::AdditionalTransactionInfo) => "Some card [456] unknown payee".to_string(),
                },
            },
            TestEntity {
                camt_txn_code: None,
                str_fields: hashmap! {
                    StrField::Camt(CamtStrField::AdditionalTransactionInfo) => "unrelated".to_string(),
                },
            },
        ];
        let want = [
            Fragment {
                cleared: true,
                account: Some("Expenses:Grocery"),
                payee: Some("Grocery shop"),
                ..Fragment::default()
            },
            Fragment {
                cleared: true,
                account: Some("Expenses:Grocery"),
                payee: Some("Another shop"),
                ..Fragment::default()
            },
            Fragment {
                cleared: false,
                account: Some("Expenses:Petrol"),
                payee: Some("Certain Petrol"),
                code: Some("123"),
                conversion: Some(&conversion),
            },
            Fragment {
                cleared: false,
                account: None,
                payee: Some("unknown payee"),
                code: Some("456"),
                ..Fragment::default()
            },
            Fragment::default(),
        ];

        let extractor = Extractor::try_new(&TestFormat::all(), &rw).unwrap();
        let got: Vec<Fragment> = input.iter().map(|t| extractor.extract(t)).collect();
        assert_eq!(want, got.as_slice());
    }
}
