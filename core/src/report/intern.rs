//! `intern` gives the String intern library combined with Bump allocator.

use std::{collections::HashSet, hash::Hash, marker::PhantomData};

use bumpalo::Bump;

/// `&str` for accounts, interned within the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Account<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for Account<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }
}

impl<'arena> Account<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// `Interner` for `Account`.
pub(super) type AccountStore<'arena> = Interner<'arena, Account<'arena>>;

/// `&str` for commodities, interned within the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Commodity<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for Commodity<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }
}

impl<'arena> Commodity<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// `Interner` for `Commodity`.
pub(super) type CommodityStore<'arena> = Interner<'arena, Commodity<'arena>>;

/// Internal type to wrap `&str` to be clear about interning.
/// Equality is compared over it's pointer, not the content.
#[derive(Debug, Eq, Clone, Copy)]
struct InternedStr<'arena>(&'arena str);

impl<'arena> InternedStr<'arena> {
    fn as_str(&self) -> &'arena str {
        self.0
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
trait FromInterned<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self;
}

pub(super) struct Interner<'arena, T> {
    idx: HashSet<&'arena str>,
    allocator: &'arena Bump,
    phantom: PhantomData<T>,
}

#[allow(private_bounds)]
impl<'arena, T: FromInterned<'arena>> Interner<'arena, T> {
    pub fn new(allocator: &'arena Bump) -> Self {
        Self {
            idx: HashSet::new(),
            allocator,
            phantom: PhantomData,
        }
    }

    /// Interns the given str and returns the shared instance.
    pub fn intern(&mut self, value: &str) -> T {
        if let Some(found) = self.get(value) {
            return found;
        }
        let copied: &'arena str = self.allocator.alloc_str(value);
        self.idx.insert(copied);
        return <T as FromInterned>::from_interned(InternedStr(copied));
    }

    /// Returns the specified value if found, otherwise None.
    pub fn get(&self, value: &str) -> Option<T> {
        self.idx
            .get(value)
            .map(|x| <T as FromInterned>::from_interned(InternedStr(x)))
    }

    /// Returns the Iterator for all elements.
    pub fn iter(&'arena self) -> Iter<'arena, T> {
        Iter {
            base: self.idx.iter(),
            phantom: PhantomData,
        }
    }
}

/// an iterator over the items of a `Interner`.
/// Compared to the underlying HashSet iterator,
/// this struct ensures the `T` type.
pub struct Iter<'arena, T> {
    base: std::collections::hash_set::Iter<'arena, &'arena str>,
    phantom: PhantomData<T>,
}

impl<'arena, T> Iterator for Iter<'arena, T>
where
    T: FromInterned<'arena>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.base
            .next()
            .map(|x| <T as FromInterned>::from_interned(InternedStr(x)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq, Eq)]
    struct TestInterned<'arena>(InternedStr<'arena>);

    impl<'arena> FromInterned<'arena> for TestInterned<'arena> {
        fn from_interned(v: InternedStr<'arena>) -> Self {
            Self(v)
        }
    }

    #[test]
    fn interner_gives_distinct_strings() {
        let bump = Bump::new();
        let mut interner = Interner::new(&bump);
        let foo: TestInterned = interner.intern("foo");
        let bar: TestInterned = interner.intern("bar");
        assert_ne!(foo, bar);
        assert_eq!(foo.0.as_str(), "foo");
        assert_eq!(bar.0.as_str(), "bar");
    }

    #[test]
    fn interner_gives_same_obj() {
        let bump = Bump::new();
        let mut interner = Interner::new(&bump);
        let foo1: TestInterned = interner.intern("foo");
        let foo2: TestInterned = interner.intern("foo");
        assert!(std::ptr::eq(foo1.0 .0, foo2.0 .0));
        assert_eq!(foo1, foo2);
        assert_eq!(foo2.0.as_str(), "foo");
    }

    #[test]
    fn interner_iter_all_elements() {
        let bump = Bump::new();
        let mut interner = Interner::new(&bump);
        let v1: TestInterned = interner.intern("abc");
        let v2: TestInterned = interner.intern("def");
        interner.intern("def");
        let v3: TestInterned = interner.intern("ghi");
        let want = vec![v1, v2, v3];

        let mut got: Vec<TestInterned> = interner.iter().collect();
        got.sort_by_key(|x| x.0.as_str());

        assert_eq!(want, got);
    }
}
