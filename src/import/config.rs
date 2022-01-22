use std::collections::HashMap;
use std::path::Path;

use log::warn;
use serde::de::Error;
use serde::{Deserialize, Serialize};

/// Set of config covering several paths.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSet {
    pub entries: Vec<ConfigEntry>,
}

impl ConfigSet {
    pub fn select(&self, p: &Path) -> Option<&ConfigEntry> {
        let fp: &str = match p.to_str() {
            None => {
                warn!("invalid Unicode path: {}", p.display());
                None
            }
            Some(x) => Some(x),
        }?;
        fn has_matches<'a>(entry: &'a ConfigEntry, fp: &str) -> Option<(usize, &'a ConfigEntry)> {
            if fp.contains(&entry.path) {
                Some((entry.path.len(), entry))
            } else {
                None
            }
        }
        self.entries
            .iter()
            .filter_map(|x| has_matches(x, fp))
            .max_by_key(|x| x.0)
            .map(|x| x.1)
    }
}

/// One entry corresponding to particular file.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub path: String,
    pub encoding: Encoding,
    pub account: String,
    pub account_type: AccountType,
    /// Operator of the import target.
    /// Required only when some charges happen.
    pub operator: Option<String>,
    pub commodity: String,
    #[serde(default)]
    pub format: FormatSpec,
    #[serde(default)]
    pub rewrite: Vec<RewriteRule>,
}

#[derive(Debug, PartialEq)]
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
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldPos {
    Index(usize),
    Label(String),
}

/// RewriteRule specifies the rewrite rule matched against transaction.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommodityConversion {
    Unspecified(UnspecifiedCommodityConversion),
    /// Specified in the rewrite rule.
    Specified {
        commodity: String,
    },
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum UnspecifiedCommodityConversion {
    Primary,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RewriteMatcher {
    Or(Vec<FieldMatcher>),
    Field(FieldMatcher),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
    UltimateCreditorName,
    DebtorName,
    UltimateDebtorName,
    RemittanceUnstructuredInfo,
    AdditionalTransactionInfo,
    Category,
    Payee,
}

use super::error::ImportError;

/// Loads Config object from given path as YAML encoded file.
pub fn load_from_yaml<R: std::io::Read>(r: R) -> Result<ConfigSet, ImportError> {
    let mut entries = Vec::new();
    let docs = serde_yaml::Deserializer::from_reader(r);
    for doc in docs {
        let entry = ConfigEntry::deserialize(doc)?;
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

    /// Create minimal ConfigEntry for testing ConfigSet::select.
    fn create_config_entry(path: &str) -> ConfigEntry {
        ConfigEntry {
            account: "Account".to_owned(),
            encoding: Encoding(encoding_rs::UTF_8),
            path: path.to_owned(),
            account_type: AccountType::Asset,
            operator: None,
            commodity: "JPY".to_owned(),
            format: FormatSpec {
                date: "%Y%m%d".to_owned(),
                commodity: HashMap::new(),
                fields: hashmap! {},
            },
            rewrite: vec![],
        }
    }

    #[test]
    fn test_config_select_single_match() {
        let config_set = ConfigSet {
            entries: vec![
                create_config_entry("/path/to/foo"),
                create_config_entry("/path/to/bar"),
            ],
        };
        assert_eq!(
            Some(&config_set.entries[0]),
            config_set.select(Path::new("/path/to/foo/202109.csv")),
        );
    }

    #[test]
    fn test_config_select_multi_match() {
        let config_set = &ConfigSet {
            entries: vec![
                create_config_entry("/path/to/foo"),
                create_config_entry("/path/to"),
            ],
        };
        assert_eq!(
            Some(&config_set.entries[0]),
            config_set.select(Path::new("/path/to/foo/202109.csv"))
        );
    }

    #[test]
    fn test_config_select_no_match() {
        let config_set = ConfigSet {
            entries: vec![
                create_config_entry("/path/to/foo"),
                create_config_entry("/path/to/bar"),
            ],
        };
        assert_eq!(
            None,
            config_set.select(Path::new("/path/to/baz/202109.csv"))
        );
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
        assert_eq!(config.entries[0].account, "Assets:Banks:Okane");
        let date = config.entries[0]
            .format
            .fields
            .get(&FieldKey::Date)
            .unwrap();
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
        assert_eq!(config.entries[0].account, "Liabilities:OkaneCard");
        let field_amount = config.entries[0]
            .format
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
