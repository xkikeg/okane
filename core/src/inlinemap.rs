use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash, RandomState},
};

use derive_where::derive_where;

use crate::fixedvec::FixedVec;

// TODO: Allow specifying N. for now let's say one, as it's cumbersome to support.
#[derive(Debug, Clone)]
#[derive_where(Default)]
#[derive_where(PartialEq; K: Eq + Hash, V: PartialEq, S: BuildHasher)]
#[derive_where(Eq; K: Eq + Hash, V: Eq, S: BuildHasher)]
pub struct InlineMap<K, V, S = RandomState> {
    inner: InlineMapImpl<K, V, S>,
}

impl<K, V> InlineMap<K, V, RandomState> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            inner: InlineMapImpl::with_capacity(size),
        }
    }
}

#[derive(Debug, Clone)]
#[derive_where(Default)]
#[derive_where(PartialEq; K: Eq + Hash, V: PartialEq, S: BuildHasher)]
#[derive_where(Eq; K: Eq + Hash, V: Eq, S: BuildHasher)]
enum InlineMapImpl<K, V, S = RandomState> {
    #[derive_where(default)]
    Small(Option<(K, V)>),
    Large(HashMap<K, V, S>),
}

impl<K, V> InlineMapImpl<K, V, RandomState> {
    fn with_capacity(size: usize) -> Self {
        if size <= 1 {
            Self::Small(None)
        } else {
            Self::Large(HashMap::with_capacity(size))
        }
    }
}
impl<K, V, S> InlineMapImpl<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let o = match self {
            Self::Large(m) => return m.insert(k, v),
            Self::Small(o) => o,
        };
        let (k1, _) = match o {
            None => {
                o.replace((k, v));
                return None;
            }
            Some(x) => x,
        };
        if k == k1 {
            o.replace((k, v)).map(|(_, val)| val)
        } else {
            *self = HashMap::from_iter([((k1,)), ()])
        }
    }
}
