use std::{
    collections::{hash_map, HashMap},
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

/// Utility macro to implement [`Merge`].
macro_rules! non_empty_right_most {
    ($a:expr, $b:expr) => {
        if !$b.is_empty() {
            $b
        } else {
            $a
        }
    };
}

pub(super) use non_empty_right_most;
