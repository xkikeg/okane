//! `intern` gives the String intern library combined with Bump allocator.

use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use bumpalo::Bump;

/// Internal type to wrap `&str` to be clear about interning.
/// Equality is compared over it's pointer, not the content.
#[derive(Eq, Clone, Copy)]
pub(super) struct InternedStr<'arena>(&'arena str);

impl<'arena> InternedStr<'arena> {
    pub fn as_str(&self) -> &'arena str {
        self.0
    }
}

impl<'arena> Debug for InternedStr<'arena> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InternedStr")
            .field(&std::ptr::from_ref(self.0))
            .field(&self.0)
            .finish()
    }
}

/// Eq is computed on the pointer, as it's interned and must have the same pointer
/// as long as the content is the same.
/// This assumes there's only one arena & `Interner` at a time and might be wrong.
///
/// For the safety, maybe we can let `InternedStr` to have arena itself,
/// so that `PartialEq` won't accidentally compare the two different intern sets.
impl<'arena> PartialEq for InternedStr<'arena> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

/// Hash is based on the pointer, as it's interned.
impl<'arena> Hash for InternedStr<'arena> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}

/// `FromInterned` is the trait that the actual Interned object must implements.
pub(super) trait FromInterned<'arena>: Copy {
    fn from_interned(v: InternedStr<'arena>) -> Self;

    fn as_interned(&self) -> InternedStr<'arena>;
}

/// Error on InternStore operations.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum InternError {
    #[error("given alias is already registered as canonical")]
    AlreadyCanonical,
    #[error("given canonical is already registered as an alias")]
    AlreadyAlias,
}

/// Manage interned `&str` in the arena allocator.
/// This can also handles alias.
/// Semantics of alias:
/// * There are two types of interned str.
///   One is canonical, and the other is alias.
/// * It's prohibited to change account type once registered.
///   Caller can't register alias which is already registered as canonical,
///   or vice versa.
pub(super) struct InternStore<'arena, T> {
    // Contains None for canonical, Some for alias.
    records: HashMap<&'arena str, Option<&'arena str>>,
    allocator: &'arena Bump,
    phantom: PhantomData<T>,
}

/// Stored value in [InternStore].
#[derive(Debug, PartialEq, Eq)]
pub enum StoredValue<'arena, T> {
    Canonical(T),
    Alias { alias: &'arena str, canonical: T },
}

impl<'arena, T: Copy> StoredValue<'arena, T> {
    fn as_canonical(&self) -> T {
        match self {
            StoredValue::Canonical(x) => *x,
            StoredValue::Alias { canonical, .. } => *canonical,
        }
    }
}

#[allow(private_bounds)]
impl<'arena, T: FromInterned<'arena>> InternStore<'arena, T> {
    pub fn new(allocator: &'arena Bump) -> Self {
        Self {
            records: HashMap::new(),
            allocator,
            phantom: PhantomData,
        }
    }

    /// Interns given `str` and returns the shared instance.
    /// Note if `value` is registered as alias, it'll resolve to the canonical one.
    /// If `value` is not registered yet, registered as canonical value.
    pub fn ensure(&mut self, value: &str) -> T {
        match self.resolve(value) {
            Some(found) => found,
            None => self.insert(value, None),
        }
    }

    /// Inserts given `value` as always canonical.
    /// Returns the registered canonical, or error if given `value` is already registered as alias.
    pub fn insert_canonical(&mut self, value: &str) -> Result<T, InternError> {
        match self.get(value) {
            None => Ok(self.insert(value, None)),
            Some(StoredValue::Canonical(found)) => Ok(found),
            Some(StoredValue::Alias { .. }) => Err(InternError::AlreadyAlias),
        }
    }

    /// Inserts given `value` as always alias of `canonical`.
    /// Returns error if given `value` is already registered as canonical.
    pub fn insert_alias(&mut self, value: &str, canonical: T) -> Result<(), InternError> {
        match self.get(value) {
            Some(StoredValue::Canonical(_)) => Err(InternError::AlreadyCanonical),
            Some(StoredValue::Alias { .. }) => Ok(()),
            None => {
                self.insert(value, Some(canonical.as_interned().as_str()));
                Ok(())
            }
        }
    }

    /// Returns the specified value if found, otherwise None.
    pub fn resolve(&self, value: &str) -> Option<T> {
        let x = self.get(value)?;
        Some(x.as_canonical())
    }

    /// Returns the Iterator for all elements.
    pub fn iter<'a>(&'a self) -> Iter<'a, T>
    where
        'a: 'arena,
    {
        Iter {
            base: self.records.iter(),
            phantom: PhantomData,
        }
    }

    #[inline]
    fn get(&self, value: &str) -> Option<StoredValue<'arena, T>> {
        let v = match self.records.get_key_value(value)? {
            (canonical, None) => StoredValue::Canonical(Self::as_type(canonical)),
            (alias, Some(canonical)) => StoredValue::Alias {
                alias,
                canonical: Self::as_type(canonical),
            },
        };
        Some(v)
    }

    #[inline]
    fn insert(&mut self, value: &str, canonical: Option<&'arena str>) -> T {
        let copied: &'arena str = self.allocator.alloc_str(value);
        let ret = self.records.insert(copied, canonical);
        debug_assert!(
            ret.is_none(),
            "insert must be called on non-existing key, but already found: {:#?}",
            ret
        );
        Self::as_type(copied)
    }

    #[inline]
    fn as_type(s: &'arena str) -> T {
        <T as FromInterned>::from_interned(InternedStr(s))
    }
}

/// an iterator over the items of a `Interner`.
/// Compared to the underlying HashSet iterator,
/// this struct ensures the `T` type.
pub struct Iter<'arena, T> {
    base: std::collections::hash_map::Iter<'arena, &'arena str, Option<&'arena str>>,
    phantom: PhantomData<T>,
}

fn to_interned<'arena, T: FromInterned<'arena>>(x: &'arena str) -> T {
    <T as FromInterned>::from_interned(InternedStr(x))
}

impl<'arena, T> Iterator for Iter<'arena, T>
where
    T: FromInterned<'arena>,
{
    type Item = StoredValue<'arena, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.base.next().map(|(key, value)| match value {
            None => StoredValue::Canonical(to_interned(key)),
            Some(value) => StoredValue::Alias {
                alias: key,
                canonical: to_interned(value),
            },
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct TestInterned<'arena>(InternedStr<'arena>);

    impl<'arena> FromInterned<'arena> for TestInterned<'arena> {
        fn from_interned(v: InternedStr<'arena>) -> Self {
            Self(v)
        }

        fn as_interned(&self) -> InternedStr<'arena> {
            self.0
        }
    }

    #[test]
    fn interned_str_eq_uses_ptr() {
        let bump = Bump::new();
        // Use non 'static str.
        let foo1 = InternedStr(bump.alloc_str(&String::from("foo")));
        let foo2 = InternedStr(bump.alloc_str(&String::from("foo")));
        assert_ne!(foo1, foo2);
        let foo_copy = foo1.clone();
        assert_eq!(foo1, foo_copy);
    }

    #[test]
    fn ensure_gives_distinct_strings() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo: TestInterned = intern_store.ensure("foo");
        let bar: TestInterned = intern_store.ensure("bar");
        assert_ne!(foo, bar);
        assert_eq!(foo.0.as_str(), "foo");
        assert_eq!(bar.0.as_str(), "bar");
    }

    #[test]
    fn ensure_gives_same_obj() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo1: TestInterned = intern_store.ensure("foo");
        let foo2: TestInterned = intern_store.ensure("foo");
        assert_eq!(foo1, foo2);
        assert_eq!(foo2.0.as_str(), "foo");
    }

    #[test]
    fn insert_canonical_succeeds_on_non_alias() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo1: TestInterned = intern_store.insert_canonical("foo").unwrap();
        let foo2: TestInterned = intern_store.insert_canonical("foo").unwrap();
        let bar: TestInterned = intern_store.insert_canonical("bar").unwrap();
        assert_eq!(foo1, foo2);
        assert_eq!(foo1.as_interned().as_str(), "foo");
        assert_ne!(foo1, bar);
        assert_eq!(bar.as_interned().as_str(), "bar");
    }

    #[test]
    fn insert_canonical_fails_when_already_alias() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo: TestInterned = intern_store.insert_canonical("foo").unwrap();
        intern_store.insert_alias("bar", foo).unwrap();
        assert_eq!(
            InternError::AlreadyAlias,
            intern_store.insert_canonical("bar").unwrap_err()
        );
    }

    #[test]
    fn insert_alias_works_on_unregistered_value() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo: TestInterned = intern_store.insert_canonical("foo").unwrap();
        intern_store.insert_alias("bar", foo).unwrap();
        let bar = intern_store.ensure("bar");
        assert_eq!(foo, bar);
    }

    #[test]
    fn insert_alias_works_on_already_alias_value() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo: TestInterned = intern_store.insert_canonical("foo").unwrap();
        intern_store.insert_alias("bar", foo).unwrap();
        intern_store.insert_alias("bar", foo).unwrap();
    }

    #[test]
    fn insert_alias_fails_on_canonical() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let foo: TestInterned = intern_store.insert_canonical("foo").unwrap();
        intern_store.insert_canonical("bar").unwrap();
        assert_eq!(
            InternError::AlreadyCanonical,
            intern_store.insert_alias("bar", foo).unwrap_err()
        );
    }

    #[test]
    fn intern_store_iter_all_elements() {
        let bump = Bump::new();
        let mut intern_store = InternStore::new(&bump);
        let v1: TestInterned = intern_store.ensure("abc");
        let v2: TestInterned = intern_store.ensure("def");
        intern_store.ensure("def");
        intern_store
            .insert_alias("efg", v1)
            .expect("efg must be an alias of abc");
        let v3: TestInterned = intern_store.ensure("ghi");
        let want = [
            StoredValue::Canonical(v1),
            StoredValue::Alias {
                alias: "efg",
                canonical: v1,
            },
            StoredValue::Canonical(v2),
            StoredValue::Canonical(v3),
        ];

        let mut got: Vec<StoredValue<TestInterned>> = intern_store.iter().collect();
        got.sort_by_key(|x| match x {
            StoredValue::Canonical(x) => (x.as_interned().as_str(), ""),
            StoredValue::Alias { alias, canonical } => (canonical.as_interned().as_str(), *alias),
        });

        assert_eq!(want.as_slice(), got.as_slice());
    }
}
