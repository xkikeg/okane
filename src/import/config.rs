use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::Path;

use log::warn;

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
                Some((fp.len(), entry))
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
    pub account: String,
    pub account_type: AccountType,
    pub commodity: String,
    pub format: FormatSpec,
    #[serde(default)]
    pub rewrite: Vec<RewriteRule>,
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatSpec {
    /// Specify the date format, in chrono::format::strftime compatible format.
    pub date: String,
    /// Mapping from abstracted field key to abstracted position.
    pub fields: HashMap<FieldKey, FieldPos>,
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
    /// Currency rate.
    Rate,
    /// Value exchanged into the main commodity (currency).
    Exchanged,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldPos {
    Index(i32),
    Label(String),
}

/// RewriteRule specifies the rewrite rule matched against transaction.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RewriteRule {
    // Regular expression that matches with payee. It must be the entire match.
    // It can have named pattern to replace these fields.
    // - code
    // - payee
    pub payee: String,

    // Account of the transaction matching against the rule.
    pub account: Option<String>,
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
    return Ok(ConfigSet { entries: entries });
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
            path: path.to_owned(),
            account_type: AccountType::Asset,
            commodity: "JPY".to_owned(),
            format: FormatSpec {
                date: "%Y%m%d".to_owned(),
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
                create_config_entry("/path/to"),
                create_config_entry("/path/to/foo"),
            ],
        };
        assert_eq!(
            Some(&config_set.entries[1]),
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
              - payee: Visaデビット　(?P<code>\d+)　(?P<payee>.*)
              - payee: 外貨普通預金（.*）(?:へ|より)振替
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
    }

    #[test]
    fn test_parse_csv_index_config() {
        let input = indoc! {r#"
        path: card/okanecard/
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
}
