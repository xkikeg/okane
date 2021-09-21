use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// Set of config covering several paths.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSet {
    pub entries: Vec<ConfigEntry>,
}

/// One entry corresponding to particular file.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub path: std::path::PathBuf,
    pub account: String,
    pub commodity: String,
    pub format: FormatSpec,
    #[serde(default)]
    pub rewrite: Vec<RewriteRule>,
}

/// FormatSpec describes the several format used in import target.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatSpec {
    /// Specify the date format, in chrono::format::strftime compatible format.
    pub date: String,
    /// Mapping from abstracted field key to String label or integer position.
    pub field: HashMap<FieldKey, String>,
}

/// Key represents the field abstracted way.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKey {
    /// Date of the transaction.
    Date,
    Payee,
    Note,
    Credit,
    Debit,
    Balance,
    Commodity,
    /// Currency rate
    Rate,
    /// Value exchanged into the main commodity (currency)
    Exchanged,
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
    use std::io::BufReader;

    #[test]
    fn test_parse_csv_label_config() -> Result<(), String> {
        let input = indoc! {r#"
            path: bank/sony/yen-futsu/
            account: Assets:Banks:Sony:Yen
            commodity: JPY
            format:
                date: "%Y年%m月%d日"
                field:
                    date: お取り引き日
                    payee: 摘要
                    note: 参考情報
                    credit: お預け入れ額
                    debit: お引き出し額
                    balance: 差し引き残高
        "#};
        let config =
            load_from_yaml(BufReader::new(input.as_bytes())).map_err(|e| format!("{:?}", e))?;
        if config.entries[0].account != "Assets:Banks:Sony:Yen" {
            return Err("invalid account".to_owned());
        }
        return Ok(());
    }
}
