//! Contains YAML serde representation for the config.

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};

use path_slash::PathBufExt;
use serde::de::value::MapAccessDeserializer;
use serde::{de, Deserialize, Serialize};

use crate::one_based::OneBasedIndex;

use super::error::ImportError;

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
                log::warn!("invalid Unicode path: {}", p.display());
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
    pub commodity: AccountCommoditySpec,
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
            .ok_or(ImportError::InvalidConfig("no commodity specified"))?
            .into();
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
    pub commodity: Option<AccountCommodityConfig>,
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
            rewrite,
        }
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

/// CommodityConfig contains either primary commodity string, or more complex CommoditySpec.
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
#[serde(untagged)]
enum AccountCommodityConfig {
    PrimaryCommodity(String),
    Spec(AccountCommoditySpec),
}

impl<'de> Deserialize<'de> for AccountCommodityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AccountCommodityConfigVisitor)
    }
}

struct AccountCommodityConfigVisitor;

impl<'de> de::Visitor<'de> for AccountCommodityConfigVisitor {
    type Value = AccountCommodityConfig;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string or a map specifying a AccountCommoditySpec"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(AccountCommodityConfig::PrimaryCommodity(v))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        AccountCommoditySpec::deserialize(MapAccessDeserializer::new(map))
            .map(AccountCommodityConfig::Spec)
    }
}

/// CommoditySpec describes commodity configs.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AccountCommoditySpec {
    /// Primary commodity used in the account.
    pub primary: String,
    /// Default conversion applied to all transaction, if not specified in rewrite rules.
    #[serde(default)]
    pub conversion: CommodityConversionSpec,
    /// Rename the key commodity into value.
    #[serde(default)]
    pub rename: HashMap<String, String>,
}

impl From<AccountCommodityConfig> for AccountCommoditySpec {
    fn from(value: AccountCommodityConfig) -> Self {
        match value {
            AccountCommodityConfig::PrimaryCommodity(c) => AccountCommoditySpec {
                primary: c,
                ..AccountCommoditySpec::default()
            },
            AccountCommodityConfig::Spec(spec) => spec,
        }
    }
}

/// FormatSpec describes the several format used in import target.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FormatSpec {
    /// Specify the date format, in [chrono::format::strftime] compatible format.
    #[serde(default)]
    pub date: String,
    /// Commodity (currency) styling.
    #[serde(default)]
    pub commodity: HashMap<String, CommodityFormatSpec>,
    /// Mapping from abstracted field key to abstracted position.
    #[serde(default)]
    pub fields: HashMap<FieldKey, FieldPos>,
    /// Delimiter for the CSV. Leave it empty to use default ",".
    #[serde(default)]
    pub delimiter: String,
    #[serde(default)]
    pub skip: SkipSpec,
    /// Order of the row. By default, old to new order.
    #[serde(default)]
    pub row_order: RowOrder,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct SkipSpec {
    /// The number of lines skipped at head.
    /// Only supported for CSV.
    pub head: i32,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum RowOrder {
    #[default]
    OldToNew,
    NewToOld,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
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
    /// Category of the transaction.
    Category,
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
    /// If not specified, fallback to primary commodity specified in the account.
    Commodity,
    /// Currency rate used in the statement.
    Rate,
    /// Secondary amount represents the amount in the secondary currency.
    /// Useful when the transaction exchanges amount into another commodity into the other.
    /// Detailed logic could be referred in [CommodityConversionSpec].
    SecondaryAmount,
    /// Secondary commodity corresponding the [FieldKey::SecondaryAmount].
    /// Also refer [CommodityConversionSpec] for the detialed explanation.
    SecondaryCommodity,
    /// Charge, commision or fee related to the transaction.
    Charge,
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub enum FieldPos {
    Index(OneBasedIndex),
    Label(String),
    Template(TemplateField),
}

impl<'de> Deserialize<'de> for FieldPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FieldPosVisitor)
    }
}

struct FieldPosVisitor;

impl<'de> de::Visitor<'de> for FieldPosVisitor {
    type Value = FieldPos;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a positive integer, a string or a map specifying a template"
        )
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match OneBasedIndex::from_one_based(v as usize) {
            Ok(x) => Ok(FieldPos::Index(x)),
            Err(_) => Err(E::invalid_value(
                de::Unexpected::Unsigned(v),
                &"a positive column index",
            )),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FieldPos::Label(v))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        TemplateField::deserialize(MapAccessDeserializer::new(map)).map(FieldPos::Template)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TemplateField {
    pub template: String,
}

/// RewriteRule specifies the rewrite rule matched against transaction.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
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
    ///
    /// This field is only used in CSV import, and only applicable when
    /// the transcation has multi commodities. See details for [CommodityConversionSpec].
    #[serde(default)]
    pub conversion: Option<CommodityConversionSpec>,
}

/// Specify the currency conversion details described in the transaction.
///
/// This is useful when CSV has non-straightforward logic.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct CommodityConversionSpec {
    /// Decides how the `secondary_amount` is computed.
    pub amount: ConversionAmountMode,
    /// Overrides `secondary_commodity` with the given value.
    pub commodity: Option<String>,
    /// Decides `rate` meaning.
    pub rate: ConversionRateMode,
    /// Disable all conversions.
    pub disabled: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ConversionAmountMode {
    /// Extracts the `secondary_amount` from the input data field.
    #[default]
    Extract,
    /// Computes the `secondary_amount` using the specified rate.
    Compute,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ConversionRateMode {
    /// Given rate is a price of secondary commodity, e.g.
    /// `1 $secondary_commodity == $rate $commodity`.
    #[default]
    PriceOfSecondary,
    /// Given rate is a price of primary commodity, e.g.
    /// `1 $commodity == $rate $secondary_commodity`
    PriceOfPrimary,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RewriteMatcher {
    Or(Vec<FieldMatcher>),
    Field(FieldMatcher),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
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
    AdditionalEntryInfo,
    AdditionalTransactionInfo,
    SecondaryCommodity,
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

    use crate::one_based;

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
            commodity: Some(AccountCommodityConfig::PrimaryCommodity("JPY".to_owned())),
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
                    format: Some(FormatSpec {
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
                    // Normally commodity won't be overridden by merge.
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
        assert_eq!(
            cfg,
            ConfigEntry {
                path: "bank/checking/".to_string(),
                encoding: Encoding(encoding_rs::UTF_8),
                account: "Assets:Banks:Checking".to_string(),
                account_type: AccountType::Asset,
                operator: None,
                commodity: simple_commodity_spec("CHF".to_string()),
                format: FormatSpec {
                    date: "%Y/%m/%d".to_string(),
                    ..Default::default()
                },
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
                    }
                ],
            }
        );
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
            .get(&FieldKey::Amount)
            .unwrap();
        assert_eq!(*field_amount, FieldPos::Index(one_based!(2)));
    }

    #[test]
    fn test_parse_template_field_pos() {
        let de = serde_yaml::Deserializer::from_str("template: \"{payee} - {category} - {note}\"");
        TemplateField::deserialize(de).expect("must not fail");

        let input = indoc! {r#"
            payee:
              template: "{commodity} - {note}"
        "#};
        let got: HashMap<FieldKey, FieldPos> = serde_yaml::from_str(input).unwrap();
        assert!(got.contains_key(&FieldKey::Payee));
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
        let matcher = RewriteRule {
            matcher: RewriteMatcher::Field(FieldMatcher {
                fields: hashmap! {RewriteField::DomainCode => "PMNT".to_string()},
            }),
            pending: false,
            payee: None,
            account: Some("Income:Salary".to_string()),
            conversion: Some(CommodityConversionSpec {
                amount: ConversionAmountMode::Extract,
                commodity: None,
                rate: ConversionRateMode::PriceOfPrimary,
                disabled: false,
            }),
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
