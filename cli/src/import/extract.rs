use super::config;
use super::error::ImportError;

use std::convert::{From, TryFrom, TryInto};

/// Extractor is a set of `ExtractRule`, so to extract `Fragment` out of the entity.
///
/// Usually entity corresponds to one transaction such as a particular CSV row.
#[derive(Debug)]
pub struct Extractor<'a, M: EntityMatcher> {
    rules: Vec<ExtractRule<'a, M>>,
}

impl<'a, M: EntityMatcher> Extractor<'a, M> {
    pub fn extract(&'a self, entity: <M as Entity<'a>>::T) -> Fragment<'a> {
        let mut fragment = Fragment::default();
        for rule in &self.rules {
            if let Some(updated) = rule.extract(fragment.clone(), entity) {
                fragment += updated;
            }
        }
        fragment
    }
}

/// Create Extractor from config.rewrite.
impl<'a, M> TryFrom<&'a Vec<config::RewriteRule>> for Extractor<'a, M>
where
    M: EntityMatcher,
{
    type Error = ImportError;
    fn try_from(rules: &'a Vec<config::RewriteRule>) -> Result<Self, Self::Error> {
        rules
            .iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<_>, _>>()
            .map(|rules| Extractor { rules })
    }
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
    pub conversion: Option<Conversion>,
}

impl<'a> std::ops::AddAssign for Fragment<'a> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: Self) {
        self.cleared = other.cleared || self.cleared;
        self.payee = other.payee.or(self.payee);
        self.account = other.account.or(self.account);
        self.code = other.code.or(self.code);
        if let Some(c) = other.conversion {
            let _ = self.conversion.insert(c);
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

/// Entity is a type of the particular `EntityMatcher` input.
///
/// Note that once GAT is available, we can instead have
///
/// type Input<'a>: Copy;
///
/// In EntityMatcher.
pub trait Entity<'a> {
    type T: Copy;
}

/// EntityGat is a pollyfil to mimic GAT like behavior without GAT.
/// Although it's public, branket impl should work all the time.
pub trait EntityGat: for<'a> Entity<'a> {}

impl<T: ?Sized> EntityGat for T where Self: for<'a> Entity<'a> {}

/// EntityMatcher defines the concrete logic for matching `Entity` and `Fragment`.
pub trait EntityMatcher:
    EntityGat + for<'a> TryFrom<(config::RewriteField, &'a str), Error = ImportError>
{
    fn captures<'a>(
        &self,
        fragment: &Fragment<'a>,
        entity: <Self as Entity<'a>>::T,
    ) -> Option<Matched<'a>>;
}

/// Matched is a result of EntityMatcher::captures method,
/// Most likely it can be regex::Capture or Matched::default().
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Matched<'a> {
    pub payee: Option<&'a str>,
    pub code: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Conversion {
    Primary,
    Specified { commodity: String },
}

impl From<config::CommodityConversion> for Conversion {
    fn from(config: config::CommodityConversion) -> Conversion {
        match config {
            config::CommodityConversion::Preset(
                config::PresetCommodityConversion::Primary,
            ) => Conversion::Primary,
            config::CommodityConversion::Trivial { commodity } => {
                Conversion::Specified { commodity }
            }
        }
    }
}

#[derive(Debug)]
struct ExtractRule<'a, M: EntityMatcher> {
    match_expr: MatchOrExpr<M>,
    pending: bool,
    payee: Option<&'a str>,
    account: Option<&'a str>,
    conversion: Option<Conversion>,
}

impl<'a, M: EntityMatcher> TryFrom<&'a config::RewriteRule> for ExtractRule<'a, M> {
    type Error = ImportError;

    fn try_from(from: &'a config::RewriteRule) -> Result<Self, Self::Error> {
        let match_expr = (&from.matcher).try_into()?;
        Ok(ExtractRule {
            match_expr,
            pending: from.pending,
            payee: from.payee.as_deref(),
            account: from.account.as_deref(),
            conversion: from.conversion.clone().map(|x| x.into()),
        })
    }
}

impl<'a, M: EntityMatcher> ExtractRule<'a, M> {
    fn extract(&self, current: Fragment<'a>, entity: <M as Entity<'a>>::T) -> Option<Fragment<'a>> {
        self.match_expr.extract(current, entity).map(|mut current| {
            current.payee = self.payee.or(current.payee);
            current.account = self.account;
            current.conversion = self.conversion.clone().or(current.conversion);
            if current.account.is_some() {
                current.cleared = current.cleared || !self.pending;
            }
            current
        })
    }
}

#[derive(Debug)]
struct MatchOrExpr<M: EntityMatcher>(Vec<MatchAndExpr<M>>);

impl<M: EntityMatcher> TryFrom<&config::RewriteMatcher> for MatchOrExpr<M> {
    type Error = ImportError;

    fn try_from(from: &config::RewriteMatcher) -> Result<Self, ImportError> {
        match from {
            config::RewriteMatcher::Or(orms) => {
                let exprs: Result<Vec<MatchAndExpr<M>>, ImportError> =
                    orms.iter().map(|x| x.try_into()).collect();
                Ok(MatchOrExpr(exprs?))
            }
            config::RewriteMatcher::Field(f) => {
                let and_expr = f.try_into()?;
                Ok(MatchOrExpr(vec![and_expr]))
            }
        }
    }
}

impl<M: EntityMatcher> MatchOrExpr<M> {
    fn extract<'a>(
        &self,
        current: Fragment<'a>,
        entity: <M as Entity<'a>>::T,
    ) -> Option<Fragment<'a>> {
        self.0
            .iter()
            .find_map(|m| m.extract(current.clone(), entity))
    }
}

#[derive(Debug)]
struct MatchAndExpr<M: EntityMatcher>(Vec<M>);

impl<M: EntityMatcher> TryFrom<&config::FieldMatcher> for MatchAndExpr<M> {
    type Error = ImportError;

    fn try_from(from: &config::FieldMatcher) -> Result<Self, ImportError> {
        let matchers: Result<Vec<M>, _> = from
            .fields
            .iter()
            .map(|(fd, v)| (*fd, v.as_str()).try_into())
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
}

impl<M: EntityMatcher> MatchAndExpr<M> {
    fn extract<'a>(
        &self,
        current: Fragment<'a>,
        entity: <M as Entity<'a>>::T,
    ) -> Option<Fragment<'a>> {
        self.0.iter().try_fold(current.clone(), |prev, matcher| {
            matcher
                .captures(&prev, entity)
                .map(|matched| prev.clone() + matched)
        })
    }
}

impl<'a> From<regex::Captures<'a>> for Matched<'a> {
    fn from(from: regex::Captures<'a>) -> Self {
        return Matched {
            payee: from.name("payee").map(|x| x.as_str()),
            code: from.name("code").map(|x| x.as_str()),
        };
    }
}

/// Creates standard regex matcher.
pub fn regex_matcher(value: &str) -> Result<regex::Regex, ImportError> {
    regex::RegexBuilder::new(value)
        .case_insensitive(true)
        .build()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[test]
    fn fragment_add_assign_filled() {
        let mut x = Fragment {
            cleared: true,
            payee: Some("foo"),
            account: None,
            code: None,
            conversion: None,
        };
        let y = Fragment {
            cleared: false,
            payee: Some("bar"),
            account: Some("baz"),
            code: Some("txn-id"),
            conversion: Some(Conversion::Primary),
        };
        x += y;
        assert_eq!(
            x,
            Fragment {
                cleared: true,
                payee: Some("bar"),
                account: Some("baz"),
                code: Some("txn-id"),
                conversion: Some(Conversion::Primary),
            }
        );
    }

    #[test]
    fn fragment_add_assign_empty() {
        let orig = Fragment {
            cleared: false,
            payee: Some("foo"),
            account: None,
            code: Some("txn-id"),
            conversion: Some(Conversion::Specified {
                commodity: "JPY".to_string(),
            }),
        };
        let mut x = orig.clone();
        x += Fragment::default();
        assert_eq!(x, orig);
    }

    #[derive(Clone, Copy)]
    struct TestEntity {
        creditor: &'static str,
        debtor: &'static str,
        additional_info: &'static str,
    }

    enum TestMatcherField {
        Creditor,
        Debtor,
        AdditionalInfo,
        Payee,
    }

    struct TestMatcher {
        field: TestMatcherField,
        pattern: regex::Regex,
    }

    impl<'a> TryFrom<(config::RewriteField, &'a str)> for TestMatcher {
        type Error = ImportError;
        fn try_from(from: (config::RewriteField, &'a str)) -> Result<Self, Self::Error> {
            let field = match from.0 {
                config::RewriteField::CreditorName => Ok(TestMatcherField::Creditor),
                config::RewriteField::DebtorName => Ok(TestMatcherField::Debtor),
                config::RewriteField::AdditionalTransactionInfo => {
                    Ok(TestMatcherField::AdditionalInfo)
                }
                config::RewriteField::Payee => Ok(TestMatcherField::Payee),
                _ => Err(ImportError::Unimplemented("no support")),
            }?;
            let pattern = regex_matcher(from.1)?;
            Ok(TestMatcher { field, pattern })
        }
    }

    impl<'a> Entity<'a> for TestMatcher {
        type T = TestEntity;
    }

    impl EntityMatcher for TestMatcher {
        fn captures<'a>(
            &self,
            fragment: &Fragment<'a>,
            entity: <Self as Entity<'a>>::T,
        ) -> Option<Matched<'a>> {
            let target = match self.field {
                TestMatcherField::Creditor => Some(entity.creditor),
                TestMatcherField::Debtor => Some(entity.debtor),
                TestMatcherField::AdditionalInfo => Some(entity.additional_info),
                TestMatcherField::Payee => fragment.payee,
            }?;
            self.pattern.captures(target).map(|x| x.into())
        }
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
    fn extract_single_match() {
        let rw = vec![config::RewriteRule {
            pending: false,
            payee: Some("Payee".to_string()),
            account: Some("Income".to_string()),
            ..into_rule(config::RewriteMatcher::Field(config::FieldMatcher {
                fields: hashmap! {
                    config::RewriteField::CreditorName => "Foo grocery".to_string(),
                    config::RewriteField::DebtorName => "Bar company".to_string(),
                },
            }))
        }];
        let input = TestEntity {
            creditor: "Foo grocery",
            debtor: "Bar company",
            additional_info: "",
        };
        let want = Fragment {
            cleared: true,
            account: Some("Income"),
            payee: Some("Payee"),
            ..Fragment::default()
        };

        let extractor: Extractor<TestMatcher> = (&rw).try_into().unwrap();
        let fragment = extractor.extract(input);

        assert_eq!(want, fragment);
    }

    #[test]
    fn extract_multi_match() {
        let rw = vec![
            config::RewriteRule {
                pending: false, // pending: true implied
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
                conversion: Some(config::CommodityConversion::Trivial {
                    commodity: "JPY".to_string(),
                }),
                ..into_rule(config::RewriteMatcher::Field(config::FieldMatcher {
                    fields: hashmap! {
                        config::RewriteField::Payee => "Certain Petrol".to_string(),
                    },
                }))
            },
        ];
        let input = vec![
            TestEntity {
                creditor: "",
                debtor: "",
                additional_info: "Some card Grocery shop",
            },
            TestEntity {
                creditor: "",
                debtor: "",
                additional_info: "Some card Another shop",
            },
            TestEntity {
                creditor: "",
                debtor: "",
                additional_info: "Some card [123] Certain Petrol",
            },
            TestEntity {
                creditor: "",
                debtor: "",
                additional_info: "Some card [456] unknown payee",
            },
            TestEntity {
                creditor: "",
                debtor: "",
                additional_info: "unrelated",
            },
        ];
        let want = vec![
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
                conversion: Some(Conversion::Specified {
                    commodity: "JPY".to_string(),
                }),
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

        let extractor: Extractor<TestMatcher> = (&rw).try_into().unwrap();
        let got: Vec<Fragment> = input
            .iter()
            .cloned()
            .map(|t| extractor.extract(t))
            .collect();
        assert_eq!(want, got);
    }
}
