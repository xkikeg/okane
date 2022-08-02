//! Contains YAML serde representation for the config.

use super::error::ImportError;

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};

use log::warn;
use path_slash::PathBufExt;
use serde::de::Error;
use serde::{Deserialize, Serialize};

/// Set of config covering several paths.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSet {
    /// Sequence of config entry.
    entries: Vec<ConfigFragment>,
}

impl ConfigSet {
    pub fn select(&self, p: &Path) -> Result<Option<ConfigEntry>, ImportError> {
        self.select_impl(p).transpose()
    }

    fn select_impl(&self, p: &Path) -> Option<Result<ConfigEntry, ImportError>> {
        let fp: &str = match p.to_str() {
            None => {
                warn!("invalid Unicode path: {}", p.display());
                None
            }
            Some(x) => Some(x),
        }?;
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
#[derive(Debug, PartialEq, Clone)]
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
    pub commodity: String,
    pub format: FormatSpec,
    pub rewrite: Vec<RewriteRule>,
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
            .ok_or(ImportError::InvalidConfig("no commodity specified"))?;
        let format = value.format.unwrap_or_default();
        Ok(ConfigEntry {
            path: value.path,
            encoding,
            account,
            account_type,
            operator: value.operator,
            commodity,
            format,
            rewrite: value.rewrite,
        })
    }
}

/// One flagment of config corresponding to particular path prefix.
/// It'll be merged into `ConfigEntry`.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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
    pub commodity: Option<String>,
    pub format: Option<FormatSpec>,
    #[serde(default)]
    pub rewrite: Vec<RewriteRule>,
}

impl ConfigFragment {
    /// Merges `other` into `self`, with overwriting as much as possible.
    /// Note `rewrite` rules will be appended.
    /// For now, `format` is also overwritten, not merged.
    fn merge(self, mut other: ConfigFragment) -> ConfigFragment {
        let mut rewrite = self.rewrite;
        rewrite.append(&mut other.rewrite);
        ConfigFragment {
            path: other.path,
            encoding: other.encoding.or(self.encoding),
            account: other.account.or(self.account),
            account_type: other.account_type.or(self.account_type),
            operator: other.operator.or(self.operator),
            commodity: other.commodity.or(self.commodity),
            format: other.format.or(self.format),
            rewrite: rewrite,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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
            .ok_or_else(|| D::Error::custom(format!("unknown encoding {}", s)))
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

/// FormatSpec describes the several format used in import target.
#[derive(Default, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FormatSpec {
    /// Specify the date format, in chrono::format::strftime compatible format.
    #[serde(default)]
    pub date: String,
    /// Commodity (currency) styling
    #[serde(default)]
    pub commodity: HashMap<String, CommodityFormatSpec>,
    /// Mapping from abstracted field key to abstracted position.
    #[serde(default)]
    pub fields: HashMap<FieldKey, FieldPos>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommodityFormatSpec {
    pub precision: u8,
}

/// Key represents the field abstracted way.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKey {
    /// Date of the transaction.
    Date,
    /// Payee, opposite side of the transaction.
    Payee,
    /// Side note.
    Note,
    /// Amount of the transcation, which can be positive or negative.
    Amount,
    /// Amount increasing the total balance.
    Credit,
    /// Amount decreasing the total balance.
    Debit,
    /// Remaining balance amount.
    Balance,
    /// Currency (commodity) of the transaction.
    Commodity,
    /// Currency rate used in the statement.
    Rate,
    /// Equivalent absolute amount exchanged into the account currency.
    /// This is the case when your account statement always shows the converted amount
    /// in the primary currency.
    EquivalentAbsolute,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum FieldPos {
    Index(usize),
    Label(String),
}

/// RewriteRule specifies the rewrite rule matched against transaction.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RewriteRule {
    /// matcher for the rewrite.
    pub matcher: RewriteMatcher,

    /// Set true to leave the match pending.
    #[serde(default)]
    pub pending: bool,

    /// Payee to be set for the matched transaction.
    #[serde(default)]
    pub payee: Option<String>,

    /// Account to be set for the matched transaction.
    #[serde(default)]
    pub account: Option<String>,

    /// Commodity (currency) conversion specification.
    #[serde(default)]
    pub conversion: Option<CommodityConversion>,
}

/// Specify what kind of the conversion is done in the transaction.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CommodityConversion {
    Unspecified(UnspecifiedCommodityConversion),
    /// Specified in the rewrite rule.
    Specified {
        commodity: String,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum UnspecifiedCommodityConversion {
    Primary,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RewriteMatcher {
    Or(Vec<FieldMatcher>),
    Field(FieldMatcher),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FieldMatcher {
    pub fields: HashMap<RewriteField, String>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RewriteField {
    DomainCode,
    DomainFamily,
    DomainSubFamily,
    CreditorName,
    CreditorAccountId,
    UltimateCreditorName,
    DebtorName,
    DebtorAccountId,
    UltimateDebtorName,
    RemittanceUnstructuredInfo,
    AdditionalTransactionInfo,
    Category,
    Payee,
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
    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

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
            commodity: Some("JPY".to_owned()),
            format: Some(FormatSpec {
                date: "%Y%m%d".to_owned(),
                ..FormatSpec::default()
            }),
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

    #[test]
    fn test_config_select_multi_match() {
        let config_set = &ConfigSet {
            entries: vec![
                create_config_fragment("path/to/foo"),
                create_config_fragment("path/to"),
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
    fn test_config_select_merge_match() {
        // TODO: write test to cover this case.
        // let config_set = ConfigSet {
        //     entries: vec![
        //         ConfigFragment {
        //             // root
        //             path: "".to_string(),
        //             encoding: Some(Encoding(encoding_rs::UTF_8)),
        //             account_type: Some(AccountType::Asset),
        //             commodity: Some("JPY".to_string()),
        //             format: Some(FormatSpec {
        //                 date: "%Y/%m/%d".to_string(),
        //                 ..Default::default()
        //             }),
        //             ..create_empty_config_fragment()
        //         },
        //     ],
        // };
    }

    #[test]
    fn test_config_entry_try_from() {
        let f = create_config_fragment("tmp/foo");
        let tryinto = |x| TryInto::<ConfigEntry>::try_into(x);
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
            ..f.clone()
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
            commodity: Some("FOO COMMODITY".to_string()),
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
                payee: 摘要
                note: 参考情報
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
            .get(&FieldKey::Date)
            .expect("format.fields.date should exist");
        assert_eq!(*date, FieldPos::Label("お取り引き日".to_owned()));
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
                conversion: Some(CommodityConversion::Specified {
                    commodity: "EUR".to_string(),
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
            date: 0
            payee: 1
            note: 6
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
            .get(&FieldKey::Amount)
            .unwrap();
        assert_eq!(*field_amount, FieldPos::Index(2));
    }

    #[test]
    fn test_parse_matcher() {
        let input = indoc! {r#"
        matcher:
          domain_code: PMNT
        account: Income:Salary
        conversion:
          type: primary
        "#};
        let de = serde_yaml::Deserializer::from_str(input);
        let matcher = RewriteRule {
            matcher: RewriteMatcher::Field(FieldMatcher {
                fields: hashmap! {RewriteField::DomainCode => "PMNT".to_string()},
            }),
            pending: false,
            payee: None,
            account: Some("Income:Salary".to_string()),
            conversion: Some(CommodityConversion::Unspecified(
                UnspecifiedCommodityConversion::Primary,
            )),
        };
        assert_eq!(matcher, RewriteRule::deserialize(de).unwrap());
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
