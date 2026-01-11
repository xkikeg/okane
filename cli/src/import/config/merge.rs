use std::{
    collections::{HashMap, hash_map},
    hash::Hash,
};

/// Merge allows merging objects into one.
pub(super) trait Merge: Sized {
    /// Merges two into one, `other` takes precedence over `self`.
    fn merge(mut self, other: Self) -> Self {
        self.merge_from(other);
        self
    }

    /// Merges `other` into `self`. Value in `other` takes precedence over `self`.
    fn merge_from(&mut self, other: Self);
}

impl<T: Merge> Merge for Option<T> {
    fn merge_from(&mut self, other: Self) {
        match (self.as_mut(), other) {
            (Some(v1), Some(v2)) => v1.merge_from(v2),
            (None, Some(v)) => *self = Some(v),
            (_, None) => (),
        }
    }
}

impl<K: Eq + Hash, T: Merge> Merge for HashMap<K, T> {
    fn merge_from(&mut self, other: Self) {
        for (k, v2) in other.into_iter() {
            match self.entry(k) {
                hash_map::Entry::Occupied(mut v1) => v1.get_mut().merge_from(v2),
                hash_map::Entry::Vacant(e) => {
                    e.insert(v2);
                }
            }
        }
    }
}

/// Utility macro to implement [`Merge`],
/// so that to choose non-empty one.
macro_rules! merge_non_empty {
    ($a:expr, $b:expr) => {
        if !$b.is_empty() { $b } else { $a }
    };
}

pub(super) use merge_non_empty;

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq, Eq)]
    struct MergeStr(&'static str);

    impl Merge for MergeStr {
        fn merge_from(&mut self, other: Self) {
            if !other.0.is_empty() {
                self.0 = other.0;
            }
        }
    }

    #[test]
    fn merge_option() {
        assert_eq!(None::<MergeStr>, None.merge(None));

        assert_eq!(Some(MergeStr("foo")), None.merge(Some(MergeStr("foo"))));

        assert_eq!(Some(MergeStr("foo")), Some(MergeStr("foo")).merge(None));

        assert_eq!(
            Some(MergeStr("bar")),
            Some(MergeStr("foo")).merge(Some(MergeStr("bar")))
        );
    }

    #[test]
    fn merge_hash_map() {
        let m1 = hashmap! {
            "key_override" => MergeStr("foo-original"),
            "key_as_is" => MergeStr("foo-as-is"),
            "key_left" => MergeStr("bar"),
        };
        let m2 = hashmap! {
            "key_override" => MergeStr("foo-override"),
            "key_as_is" => MergeStr(""),
            "key_new" => MergeStr("baz"),
        };

        let want = hashmap! {
            "key_override" => MergeStr("foo-override"),
            "key_as_is" => MergeStr("foo-as-is"),
            "key_left" => MergeStr("bar"),
            "key_new" => MergeStr("baz"),
        };

        assert_eq!(want, m1.merge(m2));
    }
}
