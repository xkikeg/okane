//! Collects all tags declared in the given Ledger file.

use std::borrow::Borrow;
use std::collections::BTreeSet;

use crate::load;
use crate::syntax::{self, plain::LedgerStatement};

use super::error::ReportError;

/// Controls whether [`tags`] reports tag keys only, or key-value pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagQuery {
    /// Report the tag key only, regardless of the value.
    KeysOnly,
    /// Report the tag as `key: value`, or the key alone if it has no value.
    WithValues,
}

/// Result of the [`tags()`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    /// Tag value itself. For key-value metadata, this is the key part.
    pub key: String,
    /// Associated value for key-value metadata.
    /// Note that the value will be always empty if the query is [`TagQuery::KeysOnly`].
    pub value: Option<TagValue>,
}

/// Value of the [`Tag`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagValue {
    /// Expression tag value. Currently we treat it as just text.
    Expr(String),
    /// Plain text value.
    Text(String),
}

impl From<&syntax::MetadataValue<'_>> for TagValue {
    fn from(value: &syntax::MetadataValue<'_>) -> Self {
        match value {
            syntax::MetadataValue::Expr(expr) => Self::Expr(expr.clone().into_owned()),
            syntax::MetadataValue::Text(text) => Self::Text(text.clone().into_owned()),
        }
    }
}

/// Returns all tags in the given Ledger file, sorted and deduped.
///
/// Tags are collected from `apply tag` directives, transaction level metadata
/// and posting level metadata. Note `apply tag` is reported as-is, and not
/// propagated into the transactions it encloses.
/// WARNING: interface are subject to change.
pub fn tags<L, F>(loader: L, query: TagQuery) -> Result<BTreeSet<Tag>, ReportError>
where
    L: Borrow<load::Loader<F>>,
    F: load::FileSystem,
{
    let mut collected: BTreeSet<Tag> = BTreeSet::new();
    loader.borrow().load(|_path, _pctx, entry| {
        match &entry.statement {
            LedgerStatement::ApplyTag(apply) => {
                insert(&mut collected, query, &apply.key, apply.value.as_ref())
            }
            LedgerStatement::Txn(txn) => {
                for metadata in &txn.metadata {
                    collect_metadata(&mut collected, query, metadata);
                }
                for posting in &txn.posts {
                    for metadata in &posting.metadata {
                        collect_metadata(&mut collected, query, metadata);
                    }
                }
            }
            _ => (),
        }
        Ok::<(), ReportError>(())
    })?;
    Ok(collected)
}

fn collect_metadata(collected: &mut BTreeSet<Tag>, query: TagQuery, metadata: &syntax::Metadata) {
    match metadata {
        syntax::Metadata::WordTags(word_tags) => {
            for tag in word_tags {
                insert(collected, query, tag, None);
            }
        }
        syntax::Metadata::KeyValueTag { key, value } => {
            insert(collected, query, key, Some(value));
        }
        syntax::Metadata::Comment(_) => (),
    }
}

fn insert(
    collected: &mut BTreeSet<Tag>,
    query: TagQuery,
    key: &str,
    value: Option<&syntax::MetadataValue>,
) {
    match (query, value) {
        (TagQuery::WithValues, Some(value)) => collected.insert(Tag {
            key: key.to_owned(),
            value: Some(value.into()),
        }),
        _ => collected.insert(Tag {
            key: key.to_owned(),
            value: None,
        }),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use indoc::indoc;
    use maplit::{btreeset, hashmap};
    use pretty_assertions::assert_eq;

    fn ko(key: &str) -> Tag {
        Tag {
            key: key.to_owned(),
            value: None,
        }
    }

    fn kv(key: &str, value: &str) -> Tag {
        Tag {
            key: key.to_owned(),
            value: Some(TagValue::Text(value.to_owned())),
        }
    }

    fn kve(key: &str, value: &str) -> Tag {
        Tag {
            key: key.to_owned(),
            value: Some(TagValue::Expr(value.to_owned())),
        }
    }

    fn loader(
        files: std::collections::HashMap<PathBuf, Vec<u8>>,
    ) -> load::Loader<load::FakeFileSystem> {
        load::Loader::new(
            PathBuf::from("path/to/root.ledger"),
            load::FakeFileSystem::from(files),
        )
    }

    fn all_tags_fixture() -> load::Loader<load::FakeFileSystem> {
        loader(hashmap! {
            PathBuf::from("path/to/root.ledger") => indoc! {"
                apply tag Workflow
                apply tag trip: kyoto

                2026/01/01 lunch
                   ; :food:trip:
                   ; type: reimbursable
                   ; just a comment, not a tag
                   Expenses:Food     10 CHF
                     ; trip: hokkaido-2026
                   Assets:Bank:X
                end apply tag
            "}.as_bytes().to_vec(),
        })
    }

    #[test]
    fn tags_keys_only() {
        let got = tags(all_tags_fixture(), TagQuery::KeysOnly).unwrap();

        assert_eq!(
            btreeset![ko("Workflow"), ko("food"), ko("trip"), ko("type")],
            got
        );
    }

    #[test]
    fn tags_with_values() {
        let got = tags(all_tags_fixture(), TagQuery::WithValues).unwrap();

        assert_eq!(
            btreeset![
                ko("Workflow"),
                ko("food"),
                ko("trip"),
                kv("trip", "hokkaido-2026"),
                kv("trip", "kyoto"),
                kv("type", "reimbursable"),
            ],
            got,
        );
    }

    #[test]
    fn tags_with_expr_value() {
        let loader = loader(hashmap! {
            PathBuf::from("path/to/root.ledger") => indoc! {"
                2026/01/01 lunch
                   Expenses:Food     10 CHF
                     ; total:: 1 + 2
                   Assets:Bank:X
            "}.as_bytes().to_vec(),
        });

        assert_eq!(
            btreeset![ko("total")],
            tags(&loader, TagQuery::KeysOnly).unwrap()
        );
        assert_eq!(
            btreeset![kve("total", "1 + 2")],
            tags(&loader, TagQuery::WithValues).unwrap(),
        );
    }

    #[test]
    fn tags_dedups_across_includes() {
        let loader = loader(hashmap! {
            PathBuf::from("path/to/root.ledger") => indoc! {"
                include child.ledger

                2026/01/01 lunch
                   ; :food:
                   ; trip: kyoto
                   Expenses:Food     10 CHF
                   Assets:Bank:X
            "}.as_bytes().to_vec(),
            PathBuf::from("path/to/child.ledger") => indoc! {"
                2026/01/02 dinner
                   ; :food:
                   ; trip: kyoto
                   Expenses:Food     20 CHF
                   Assets:Bank:X
            "}.as_bytes().to_vec(),
        });

        assert_eq!(
            btreeset![ko("food"), ko("trip")],
            tags(&loader, TagQuery::KeysOnly).unwrap()
        );
        assert_eq!(
            btreeset![ko("food"), kv("trip", "kyoto")],
            tags(&loader, TagQuery::WithValues).unwrap(),
        );
    }
}
