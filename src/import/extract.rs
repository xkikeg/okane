use super::config;
use super::error::ImportError;

use std::convert::{From, TryFrom, TryInto};

#[derive(Debug)]
pub struct Extractor<'a, M: EntityMatcher> {
    rules: Vec<ExtractRule<'a, M>>,
}

impl<'a, M: EntityMatcher> Extractor<'a, M> {
    pub fn extract(&'a self, entity: <M as Entity<'a>>::T) -> Fragment<'a> {
        let mut fragment = Fragment {
            cleared: false,
            payee: None,
            account: None,
        };
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

/// Entity declares what is the input entity for the EntityMatcher trait,
/// as GATs is not yet stable.
/// Once it's available in stable, we can instead have
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

/// EntityMatcher defines how to match a given Entity against the config.
pub trait EntityMatcher: EntityGat +
    for<'a> TryFrom<(config::RewriteField, &'a str), Error = ImportError>
{
    fn captures<'a>(
        &self,
        fragment: &Fragment<'a>,
        entity: <Self as Entity<'a>>::T,
    ) -> Option<Matched<'a>>;
}

/// Matched is a result of EntityMatcher::captures method,
/// Most likely it can be regex::Capture or Matched::empty().
#[derive(Debug, Default)]
pub struct Matched<'a> {
    pub payee: Option<&'a str>,
}

#[derive(Debug)]
struct ExtractRule<'a, M: EntityMatcher> {
    match_expr: MatchOrExpr<M>,
    pending: bool,
    payee: Option<&'a str>,
    account: Option<&'a str>,
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
        })
    }
}

impl<'a, M: EntityMatcher> ExtractRule<'a, M> {
    fn extract(&self, current: Fragment<'a>, entity: <M as Entity<'a>>::T) -> Option<Fragment<'a>> {
        self.match_expr.extract(current, entity).map(|mut current| {
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
    fn extract<'a>(&self, current: Fragment<'a>, entity: <M as Entity<'a>>::T) -> Option<Fragment<'a>> {
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
    fn extract<'a>(&self, current: Fragment<'a>, entity: <M as Entity<'a>>::T) -> Option<Fragment<'a>> {
        fn extract_impl<'a, M>(
            m: &M,
            fragment: Fragment<'a>,
            entity: <M as Entity<'a>>::T,
        ) -> Option<Fragment<'a>>
        where
            M: EntityMatcher,
        {
            match m.captures(&fragment, entity) {
                None => None,
                Some(m) => {
                    let mut fragment = fragment.clone();
                    if let Some(p) = m.payee {
                        fragment.payee = Some(p);
                    }
                    Some(fragment)
                }
            }
        }
        self.0
            .iter()
            .try_fold(current.clone(), |prev, m| extract_impl(m, prev, entity))
    }
}

impl<'a> From<regex::Captures<'a>> for Matched<'a> {
    fn from(from: regex::Captures<'a>) -> Self {
        return Matched {
            payee: from.name("payee").map(|x| x.as_str()),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

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

    use regex::Regex;

    struct TestMatcher {
        field: TestMatcherField,
        pattern: Regex,
    }

    impl<'a> TryFrom<(config::RewriteField, &'a str)> for TestMatcher {
        type Error = ImportError;
        fn try_from(from: (config::RewriteField, &'a str)) -> Result<Self, Self::Error> {
            let field = match from.0 {
                config::RewriteField::CreditorName => Ok(TestMatcherField::Creditor),
                config::RewriteField::DebtorName => Ok(TestMatcherField::Debtor),
                config::RewriteField::AdditionalTransactionInfo => Ok(TestMatcherField::AdditionalInfo),
                config::RewriteField::Payee => Ok(TestMatcherField::Payee),
                _ => Err(ImportError::Unimplemented("no support")),
            }?;
            let pattern = Regex::new(from.1)?;
            Ok(TestMatcher{field, pattern})
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
            self.pattern.captures(target).map(|c| Matched {
                payee: c.name("payee").map(|x| x.as_str()),
            })
        }
    }

    #[test]
    fn test_extract_single_match() {
        let rw = vec![config::RewriteRule {
            matcher: config::RewriteMatcher::Field(config::FieldMatcher {
                fields: hashmap! {
                    config::RewriteField::CreditorName => "Foo grocery".to_string(),
                    config::RewriteField::DebtorName => "Bar company".to_string(),
                },
            }),
            pending: false,
            payee: Some("Payee".to_string()),
            account: Some("Income".to_string()),
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
        };

        let extractor: Extractor<TestMatcher> = (&rw).try_into().unwrap();
        let fragment = extractor.extract(input);

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
                additional_info: "Some card Certain Petrol",
            },
            TestEntity {
                creditor: "",
                debtor: "",
                additional_info: "Some card unknown payee",
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

        let extractor: Extractor<TestMatcher> = (&rw).try_into().unwrap();
        let got: Vec<Fragment> = input
            .iter()
            .cloned()
            .map(|t| extractor.extract(t))
            .collect();
        assert_eq!(want, got);
    }
}
