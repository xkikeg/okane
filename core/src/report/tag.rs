//! Tags attached to transactions and postings.
//!
//! Tag keys are interned in [`TagStore`], like accounts and commodities.
//! A [`TagSet`] holds the tags of one transaction or posting, with no key
//! appearing twice.

use bumpalo::Bump;
use bumpalo_intern::direct::{DirectInternStore, FromInterned, InternedStr};

/// Key of a tag, such as `trip` in `; trip: Kyoto` or `; :trip:`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TagKey<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for TagKey<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }

    fn as_interned(&self) -> InternedStr<'arena> {
        self.0
    }
}

impl<'arena> TagKey<'arena> {
    /// Returns the `&str`.
    #[inline]
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// Manages [`TagKey`] instances.
pub struct TagStore<'arena> {
    intern: DirectInternStore<'arena, TagKey<'arena>>,
}

impl<'arena> TagStore<'arena> {
    /// Creates a new instance.
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            intern: DirectInternStore::new(arena),
        }
    }

    /// Returns the [`TagKey`] for `value`, registering it if needed.
    pub fn ensure(&mut self, value: &str) -> TagKey<'arena> {
        self.intern.ensure(value)
    }

    /// Returns the [`TagKey`] for `value` if it's already registered.
    pub fn resolve(&self, value: &str) -> Option<TagKey<'arena>> {
        self.intern.resolve(value)
    }
}

/// A tag with its optional value.
///
/// Word tags (`:foo:`) have no value. Expression values (`key:: expr`) are
/// kept as their source text for now.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TagEntry<'arena> {
    pub key: TagKey<'arena>,
    pub value: Option<&'arena str>,
}

/// Tags of a transaction or a posting, sorted by key, without duplicates.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct TagSet<'arena>(&'arena [TagEntry<'arena>]);

impl<'arena> TagSet<'arena> {
    /// Builds a [`TagSet`], or returns the first key that appears twice.
    pub(super) fn new(
        arena: &'arena Bump,
        mut entries: Vec<TagEntry<'arena>>,
    ) -> Result<Self, TagKey<'arena>> {
        if entries.is_empty() {
            return Ok(Self::default());
        }
        // Stable sort, so the reported duplicate is the later occurrence.
        entries.sort_by_key(|e| e.key.as_str());
        if let Some(dup) = entries.windows(2).find(|w| w[0].key == w[1].key) {
            return Err(dup[1].key);
        }
        Ok(Self(arena.alloc_slice_copy(&entries)))
    }

    /// Returns the tags, sorted by key.
    pub fn iter(&self) -> impl Iterator<Item = &TagEntry<'arena>> {
        self.0.iter()
    }

    /// Returns the entry for `key`, if any.
    pub fn get(&self, key: TagKey<'arena>) -> Option<&TagEntry<'arena>> {
        self.0
            .binary_search_by_key(&key.as_str(), |e| e.key.as_str())
            .ok()
            .map(|i| &self.0[i])
    }

    /// Returns `true` if there's no tag.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of tags.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn tag_set_sorted_by_key() {
        let arena = Bump::new();
        let mut store = TagStore::new(&arena);
        let b = store.ensure("b");
        let a = store.ensure("a");
        let set = TagSet::new(
            &arena,
            vec![
                TagEntry {
                    key: b,
                    value: Some("x"),
                },
                TagEntry {
                    key: a,
                    value: None,
                },
            ],
        )
        .unwrap();
        let keys: Vec<&str> = set.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(vec!["a", "b"], keys);
        assert_eq!(Some("x"), set.get(b).unwrap().value);
        assert_eq!(None, set.get(a).unwrap().value);
    }

    #[test]
    fn tag_set_rejects_duplicate() {
        let arena = Bump::new();
        let mut store = TagStore::new(&arena);
        let a = store.ensure("a");
        let b = store.ensure("b");
        let got = TagSet::new(
            &arena,
            vec![
                TagEntry {
                    key: a,
                    value: None,
                },
                TagEntry {
                    key: b,
                    value: None,
                },
                TagEntry {
                    key: a,
                    value: Some("v"),
                },
            ],
        );
        assert_eq!(Err(a), got);
    }
}
