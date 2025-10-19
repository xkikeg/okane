//! Contains YAML serde representation for the config.

mod commodity;
mod format;
mod merge;
mod output;
mod rewrite;

pub use commodity::{
    AccountCommoditySpec, CommodityConversionSpec, ConversionAmountMode, ConversionRateMode,
};
pub use format::{FieldKey, FieldPos, FormatSpec, RowOrder, SkipSpec, TemplateField};
pub use output::{
    CommodityFormatStyle, OutputCommodityDetailsSpec, OutputCommoditySpec, OutputSpec,
};
pub use rewrite::{FieldMatcher, RewriteField, RewriteMatcher, RewriteRule};
use soft_canonicalize::soft_canonicalize;

use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};

use path_slash::PathBufExt;
use serde::{de, Deserialize, Serialize};

use super::error::ImportError;

use merge::Merge;

/// Set of config covering several paths.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSet {
    /// Sequence of config entry.
    entries: Vec<ConfigFragment>,
}

impl ConfigSet {
    pub fn select(&self, p: &Path) -> Result<Option<ConfigEntry>, ImportError> {
        self.select_impl(&soft_canonicalize(p)?).transpose()
    }

    fn select_impl(&self, p: &Path) -> Option<Result<ConfigEntry, ImportError>> {
        let fp: &str = match p.to_str() {
            None => {
                return Some(Err(ImportError::Other(format!(
                    "invalid Unicode path: {}",
                    p.display()
                ))));
            }
            Some(x) => x,
        };
        fn has_matches<'a>(
            entry: &'a ConfigFragment,
            fp: &str,
        ) -> Option<(usize, &'a ConfigFragment)> {
            let epb: PathBuf = PathBufExt::from_slash(&entry.path);
            log::trace!("epb={} fp={}", epb.display(), fp);
            match epb.to_str() {
                Some(ep) if fp.contains(ep) => Some((entry.path.len(), entry)),
                _ => None,
            }
        }
        let mut matched: Vec<(usize, &ConfigFragment)> = self
            .entries
            .iter()
            .filter_map(|x| has_matches(x, fp))
            .collect();
        matched.sort_by_key(|x| x.0);
        matched
            .into_iter()
            .fold(None, |res, item| match res {
                None => Some(item.1.clone()),
                Some(prev) => Some(prev.merge(item.1.clone())),
            })
            .map(|x| x.try_into())
    }
}

/// One entry corresponding to particular file.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConfigEntry {
    /// Path pattern which file the entry will match against.
    pub path: String,
    /// File encoding of the input.
    pub encoding: Encoding,
    /// Account name used in the ledger.
    pub account: String,
    /// Type of account input.
    pub account_type: AccountType,
    /// Operator of the import target.
    /// Required only when some charges applied.
    pub operator: Option<String>,
    /// Commodity handling about the account.
    pub commodity: commodity::AccountCommoditySpec,
    /// Format of the given input file.
    pub format: format::FormatSpec,
    /// Specification about output file.
    pub output: OutputSpec,
    pub rewrite: Vec<rewrite::RewriteRule>,
}

impl TryFrom<ConfigFragment> for ConfigEntry {
    type Error = ImportError;
    fn try_from(value: ConfigFragment) -> Result<Self, Self::Error> {
        let encoding = value
            .encoding
            .ok_or(ImportError::InvalidConfig("no encoding specified"))?;
        let account = value
            .account
            .ok_or(ImportError::InvalidConfig("no account specified"))?;
        let account_type = value
            .account_type
            .ok_or(ImportError::InvalidConfig("no account_type specified"))?;
        let commodity = value
            .commodity
            .ok_or(ImportError::InvalidConfig("no commodity specified"))?
            .into();
        let format = value.format.unwrap_or_default();
        let output = value.output.unwrap_or_default();
        Ok(ConfigEntry {
            path: value.path,
            encoding,
            account,
            account_type,
            operator: value.operator,
            commodity,
            format,
            output,
            rewrite: value.rewrite,
        })
    }
}

/// One flagment of config corresponding to particular path prefix.
/// It'll be merged into `ConfigEntry`.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ConfigFragment {
    /// Path pattern which file the entry will match against.
    pub path: String,
    /// File encoding of the input.
    pub encoding: Option<Encoding>,
    /// Account name used in the ledger.
    pub account: Option<String>,
    /// Type of account input.
    pub account_type: Option<AccountType>,
    /// Operator of the import target.
    /// Required only when some charges applied.
    pub operator: Option<String>,
    pub commodity: Option<commodity::AccountCommodityConfig>,
    pub format: Option<format::FormatSpec>,
    pub output: Option<OutputSpec>,
    #[serde(default)]
    pub rewrite: Vec<rewrite::RewriteRule>,
}

impl Merge for ConfigFragment {
    fn merge(mut self, mut other: ConfigFragment) -> Self {
        self.rewrite.append(&mut other.rewrite);
        self.commodity.merge_from(other.commodity);
        self.format.merge_from(other.format);
        self.output.merge_from(other.output);
        Self {
            path: other.path,
            encoding: other.encoding.or(self.encoding),
            account: other.account.or(self.account),
            account_type: other.account_type.or(self.account_type),
            operator: other.operator.or(self.operator),
            commodity: self.commodity,
            format: self.format,
            output: self.output,
            rewrite: self.rewrite,
        }
    }

    fn merge_from(&mut self, other: Self) {
        // insufficient implementation, but safer unless we implement derive macro.
        *self = self.clone().merge(other);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Encoding(pub &'static encoding_rs::Encoding);

impl Encoding {
    pub fn as_encoding(&self) -> &'static encoding_rs::Encoding {
        self.0
    }
}

impl Serialize for Encoding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.name().as_bytes())
    }
}

impl<'de> Deserialize<'de> for Encoding {
    fn deserialize<D>(deserializer: D) -> Result<Encoding, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        encoding_rs::Encoding::for_label(s.as_bytes())
            .ok_or_else(|| {
                use de::Error;
                D::Error::invalid_value(de::Unexpected::Str(&s), &"valid string encoding")
            })
            .map(Encoding)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// Account is an asset, +amount will increase the amount.
    Asset,
    /// Account is a liability, +amount will decrease the amount.
    Liability,
}

/// Loads Config object from given path as YAML encoded file.
pub fn load_from_yaml<R: std::io::Read>(r: R) -> Result<ConfigSet, ImportError> {
    let mut entries = Vec::new();
    let docs = serde_yaml::Deserializer::from_reader(r);
    for doc in docs {
        let entry = ConfigFragment::deserialize(doc)?;
        entries.push(entry);
    }
    Ok(ConfigSet { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::one_based_macro::one_based_32;

    use super::commodity::{AccountCommodityConfig, AccountCommoditySpec};

    #[ctor::ctor]
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Create an empty ConfigFragment.
    fn create_empty_config_fragment() -> ConfigFragment {
        ConfigFragment {
            path: "empty/".to_string(),
            account: None,
            account_type: None,
            commodity: None,
            encoding: None,
            format: None,
            output: None,
            operator: None,
            rewrite: Vec::new(),
        }
    }

    /// Create a minimal ConfigFragment to generate valid ConfigEntry.
    fn create_config_fragment(path: &str) -> ConfigFragment {
        ConfigFragment {
            account: Some("Account".to_owned()),
            encoding: Some(Encoding(encoding_rs::UTF_8)),
            path: path.to_owned(),
            account_type: Some(AccountType::Asset),
            operator: None,
            commodity: Some(AccountCommodityConfig::PrimaryCommodity("JPY".to_owned())),
            format: Some(format::FormatSpec {
                date: "%Y%m%d".to_owned(),
                ..format::FormatSpec::default()
            }),
            output: None,
            rewrite: vec![],
        }
    }

    #[test]
    fn test_config_select_single_match() {
        let config_set = ConfigSet {
            entries: vec![
                create_config_fragment("path/to/foo"),
                create_config_fragment("path/to/bar"),
            ],
        };
        let cfg0: ConfigEntry = config_set.entries[0]
            .clone()
            .try_into()
            .expect("config[0] must be valid ConfigEntry");
        assert_eq!(
            cfg0,
            config_set
                .select(&Path::new("path").join("to").join("foo").join("202109.csv"))
                .expect("select must not fail")
                .expect("select must have result")
        );
    }

    #[test]
    fn test_config_select_no_match() {
        let config_set = ConfigSet {
            entries: vec![
                create_config_fragment("path/to/foo"),
                create_config_fragment("path/to/bar"),
            ],
        };
        assert_eq!(
            None,
            config_set
                .select(&Path::new("path").join("to").join("baz").join("202109.csv"))
                .expect("select must not fail")
        );
    }

    fn simple_commodity_spec(primary: String) -> AccountCommoditySpec {
        AccountCommoditySpec {
            primary,
            conversion: CommodityConversionSpec::default(),
            rename: HashMap::new(),
        }
    }

    #[test]
    fn test_config_select_merge_match() {
        let config_set = ConfigSet {
            entries: vec![
                ConfigFragment {
                    path: "bank/".to_string(),
                    encoding: Some(Encoding(encoding_rs::UTF_8)),
                    account_type: Some(AccountType::Asset),
                    commodity: Some(AccountCommodityConfig::PrimaryCommodity("JPY".to_string())),
                    format: Some(format::FormatSpec {
                        date: "%Y/%m/%d".to_string(),
                        ..Default::default()
                    }),
                    rewrite: vec![RewriteRule {
                        matcher: RewriteMatcher::Field(FieldMatcher {
                            fields: hashmap! {
                                RewriteField::Payee => r#"foo"#.to_string(),
                            },
                        }),
                        account: Some("Account Foo".to_string()),
                        payee: None,
                        conversion: None,
                        pending: false,
                    }],
                    ..create_empty_config_fragment()
                },
                ConfigFragment {
                    path: "bank/checking/".to_string(),
                    commodity: Some(AccountCommodityConfig::Spec(simple_commodity_spec(
                        "CHF".to_string(),
                    ))),
                    account: Some("Assets:Banks:Checking".to_string()),
                    rewrite: vec![RewriteRule {
                        matcher: RewriteMatcher::Field(FieldMatcher {
                            fields: hashmap! {
                                RewriteField::Payee => r#"bar"#.to_string(),
                            },
                        }),
                        account: Some("Account Bar".to_string()),
                        payee: None,
                        conversion: None,
                        pending: false,
                    }],
                    ..create_empty_config_fragment()
                },
            ],
        };
        let cfg = config_set
            .select(
                &Path::new("path")
                    .join("to")
                    .join("bank")
                    .join("checking")
                    .join("202109.csv"),
            )
            .expect("select must not fail")
            .expect("entry should be found");
        let want = ConfigEntry {
            path: "bank/checking/".to_string(),
            encoding: Encoding(encoding_rs::UTF_8),
            account: "Assets:Banks:Checking".to_string(),
            account_type: AccountType::Asset,
            operator: None,
            commodity: simple_commodity_spec("CHF".to_string()),
            format: format::FormatSpec {
                date: "%Y/%m/%d".to_string(),
                ..Default::default()
            },
            output: OutputSpec::default(),
            rewrite: vec![
                RewriteRule {
                    matcher: RewriteMatcher::Field(FieldMatcher {
                        fields: hashmap! {
                            RewriteField::Payee => r#"foo"#.to_string(),
                        },
                    }),
                    account: Some("Account Foo".to_string()),
                    payee: None,
                    conversion: None,
                    pending: false,
                },
                RewriteRule {
                    matcher: RewriteMatcher::Field(FieldMatcher {
                        fields: hashmap! {
                            RewriteField::Payee => r#"bar"#.to_string(),
                        },
                    }),
                    account: Some("Account Bar".to_string()),
                    payee: None,
                    conversion: None,
                    pending: false,
                },
            ],
        };
        assert_eq!(want, cfg);
    }

    #[test]
    fn test_config_entry_try_from() {
        let f = create_config_fragment("tmp/foo");
        let tryinto = TryInto::<ConfigEntry>::try_into;
        tryinto(f.clone()).expect("create_config_fragment() should return solo valid fragment");
        let err = |x| tryinto(x).unwrap_err().to_string();
        assert!(err(ConfigFragment {
            account: None,
            ..f.clone()
        })
        .contains("account"));
        assert!(err(ConfigFragment {
            encoding: None,
            ..f.clone()
        })
        .contains("encoding"));
        assert!(err(ConfigFragment {
            account_type: None,
            ..f.clone()
        })
        .contains("account_type"));
        assert!(err(ConfigFragment {
            commodity: None,
            ..f
        })
        .contains("commodity"));
    }

    #[test]
    fn test_config_fragment_merge() {
        let empty = create_empty_config_fragment;
        let account = || ConfigFragment {
            path: "account/".to_string(),
            account: Some("foo".to_string()),
            ..empty()
        };
        let account_type = || ConfigFragment {
            path: "account_type/".to_string(),
            account_type: Some(AccountType::Liability),
            ..empty()
        };
        let commodity = || ConfigFragment {
            path: "commodity/".to_string(),
            commodity: Some(AccountCommodityConfig::Spec(simple_commodity_spec(
                "primary commodity".to_string(),
            ))),
            ..empty()
        };
        let operator = || ConfigFragment {
            path: "fragment/".to_string(),
            operator: Some("foo operator".to_string()),
            ..empty()
        };
        let cases: Vec<Box<dyn Fn() -> ConfigFragment>> = vec![
            Box::new(account),
            Box::new(account_type),
            Box::new(commodity),
            Box::new(operator),
        ];
        for case in cases {
            assert_eq!(empty().merge(case()), case());
            assert_eq!(
                case().merge(empty()),
                ConfigFragment {
                    path: "empty/".to_string(),
                    ..case()
                }
            );
        }
    }

    #[test]
    fn test_parse_csv_label_config() {
        let input = indoc! {r#"
            path: bank/okanebank/
            encoding: Shift_JIS
            account: Assets:Banks:Okane
            account_type: asset
            commodity: JPY
            format:
              date: "%Y年%m月%d日"
              fields:
                date: お取り引き日
                payee:
                    template: "{commodity} - {note}"
                note: 摘要
                credit: お預け入れ額
                debit: お引き出し額
                balance: 差し引き残高
            rewrite:
              - matcher:
                  payee: Visaデビット　(?P<code>\d+)　(?P<payee>.*)
              - matcher:
                  payee: 外貨普通預金（.*）(?:へ|より)振替
                conversion:
                  commodity: EUR
                account: Assets:Wire:Okane
        "#};
        let config = load_from_yaml(input.as_bytes()).unwrap();
        assert_eq!(
            config.entries[0].account.as_deref(),
            Some("Assets:Banks:Okane")
        );
        let date = config.entries[0]
            .format
            .as_ref()
            .expect("format should exist")
            .fields
            .get(&format::FieldKey::Date)
            .expect("format.fields.date should exist");
        assert_eq!(*date, format::FieldPos::Label("お取り引き日".to_owned()));
        let rewrite = vec![
            RewriteRule {
                matcher: RewriteMatcher::Field(FieldMatcher {
                    fields: hashmap! {
                        RewriteField::Payee => r#"Visaデビット　(?P<code>\d+)　(?P<payee>.*)"#.to_string(),
                    },
                }),
                pending: false,
                payee: None,
                account: None,
                conversion: None,
            },
            RewriteRule {
                matcher: RewriteMatcher::Field(FieldMatcher {
                    fields: hashmap! {
                        RewriteField::Payee => "外貨普通預金（.*）(?:へ|より)振替".to_string(),
                    },
                }),
                pending: false,
                payee: None,
                account: Some("Assets:Wire:Okane".to_string()),
                conversion: Some(CommodityConversionSpec {
                    commodity: Some("EUR".to_string()),
                    ..CommodityConversionSpec::default()
                }),
            },
        ];
        assert_eq!(&rewrite, &config.entries[0].rewrite);
    }

    #[test]
    fn test_parse_csv_index_config() {
        let input = indoc! {r#"
        path: card/okanecard/
        encoding: UTF-8
        account: Liabilities:OkaneCard
        account_type: liability
        commodity: JPY
        format:
          date: "%Y/%m/%d"
          fields:
            date: 1
            payee: 1
            note:
              template: "{category} - {payee}"
            amount: 2
        rewrite: []
        "#};
        let config = load_from_yaml(input.as_bytes()).unwrap();
        assert_eq!(
            config.entries[0].account.as_deref(),
            Some("Liabilities:OkaneCard")
        );
        let field_amount = config.entries[0]
            .format
            .as_ref()
            .expect("FormatSpec should exist")
            .fields
            .get(&format::FieldKey::Amount)
            .unwrap();
        assert_eq!(*field_amount, format::FieldPos::Index(one_based_32!(2)));
    }

    #[test]
    fn test_parse_template_field_pos() {
        let de = serde_yaml::Deserializer::from_str("template: \"{payee} - {category} - {note}\"");
        format::TemplateField::deserialize(de).expect("must not fail");

        let input = indoc! {r#"
            payee:
              template: "{commodity} - {note}"
        "#};
        let got: HashMap<format::FieldKey, format::FieldPos> = serde_yaml::from_str(input).unwrap();
        assert!(got.contains_key(&format::FieldKey::Payee));
    }

    #[test]
    fn test_parse_matcher() {
        let input = indoc! {r#"
        matcher:
          domain_code: PMNT
        account: Income:Salary
        conversion:
          rate: price_of_primary
        "#};
        let de = serde_yaml::Deserializer::from_str(input);
        let want = RewriteRule {
            matcher: RewriteMatcher::Field(FieldMatcher {
                fields: hashmap! {RewriteField::DomainCode => "PMNT".to_string()},
            }),
            pending: false,
            payee: None,
            account: Some("Income:Salary".to_string()),
            conversion: Some(CommodityConversionSpec {
                amount: None,
                commodity: None,
                rate: Some(ConversionRateMode::PriceOfPrimary),
                disabled: None,
            }),
        };
        assert_eq!(want, RewriteRule::deserialize(de).unwrap());
    }

    #[test]
    fn test_parse_isocamt_config() {
        let input = indoc! {r#"
        path: bank/okanebank/
        encoding: UTF-8
        account: Banks:Okane
        account_type: asset
        commodity: USD
        rewrite:
          - matcher:
              domain_code: PMNT
              domain_family: RCDT
              domain_sub_family: SALA
            account: Income:Salary
            payee: Okane Co. Ltd.
          - matcher:
            - payee: Migros
            - payee: Coop
            account: Expenses:Grocery
          - matcher:
              additional_transaction_info: Maestro(?P<payee>.*)
        "#};
        let config = load_from_yaml(input.as_bytes()).unwrap();
        let rewrite = vec![
            RewriteRule {
                matcher: RewriteMatcher::Field(FieldMatcher {
                    fields: hashmap! {
                        RewriteField::DomainCode => "PMNT".to_string(),
                        RewriteField::DomainFamily => "RCDT".to_string(),
                        RewriteField::DomainSubFamily => "SALA".to_string(),
                    },
                }),
                pending: false,
                payee: Some("Okane Co. Ltd.".to_string()),
                account: Some("Income:Salary".to_string()),
                conversion: None,
            },
            RewriteRule {
                matcher: RewriteMatcher::Or(vec![
                    FieldMatcher {
                        fields: hashmap! {
                            RewriteField::Payee => "Migros".to_string(),
                        },
                    },
                    FieldMatcher {
                        fields: hashmap! {
                            RewriteField::Payee => "Coop".to_string(),
                        },
                    },
                ]),
                pending: false,
                account: Some("Expenses:Grocery".to_string()),
                payee: None,
                conversion: None,
            },
            RewriteRule {
                matcher: RewriteMatcher::Field(FieldMatcher {
                    fields: hashmap! {
                        RewriteField::AdditionalTransactionInfo => "Maestro(?P<payee>.*)".to_string(),
                    },
                }),
                pending: false,
                payee: None,
                account: None,
                conversion: None,
            },
        ];
        assert_eq!(&rewrite, &config.entries[0].rewrite);
    }
}
