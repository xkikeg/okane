use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

use bumpalo::Bump;

/// Internal type to wrap `&str` to be clear about interning.
/// Equality is compared over it's pointer, not the content.
#[derive(Debug, Eq, Clone, Copy)]
pub(super) struct InternedStr<'arena>(&'arena str);

impl<'arena> InternedStr<'arena> {
    pub(super) fn as_str(&self) -> &'arena str {
        self.0
    }
}

impl<'arena> PartialEq for InternedStr<'arena> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'arena> Hash for InternedStr<'arena> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}

pub(super) trait FromInterned<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self;
}

pub struct Interner<'arena, T> {
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

    /// Returns the corresponding interned str to the given str if found,
    /// or returns None.
    pub fn get(&self, value: &str) -> Option<T> {
        self.idx
            .get(value)
            .map(|found| <T as FromInterned>::from_interned(InternedStr(found)))
    }

    /// Interns given `&str` and returns the interned str.
    pub fn intern(&mut self, value: &str) -> T {
        if let Some(found) = self.get(value) {
            return found;
        }
        let copied: &'arena str = self.allocator.alloc_str(value);
        self.idx.insert(copied);
        return <T as FromInterned>::from_interned(InternedStr(copied));
    }

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
    pub(crate) base: std::collections::hash_set::Iter<'arena, &'arena str>,
    pub(crate) phantom: PhantomData<T>,
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
pub(crate) mod tests {
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
}
