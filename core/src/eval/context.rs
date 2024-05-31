use std::{collections::HashSet, marker::PhantomData};

use bumpalo::Bump;

#[derive(Debug, Eq)]
pub struct Interned<'bump, Tag>(&'bump str, PhantomData<Tag>);

impl<'bump, Tag> PartialEq for Interned<'bump, Tag> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'bump, Tag> Interned<'bump, Tag> {
    pub fn as_str(&'bump self) -> &'bump str {
        return self.0;
    }
}

pub struct Interner<'bump, Tag> {
    idx: HashSet<&'bump str>,
    allocator: &'bump Bump,
    phantom: PhantomData<Tag>,
}

impl<'bump, Tag> Interner<'bump, Tag> {
    pub fn new(allocator: &'bump Bump) -> Self {
        Self {
            idx: HashSet::new(),
            allocator: allocator,
            phantom: PhantomData,
        }
    }

    pub fn from_str(&mut self, value: &str) -> Interned<'bump, Tag> {
        if let Some(found) = self.idx.get(value) {
            return Interned(*found, self.phantom);
        }
        let copied: &'bump str = self.allocator.alloc_str(value);
        self.idx.insert(copied);
        return Interned(copied, self.phantom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[derive(Debug)]
    struct FakeTag();

    #[test]
    fn interner_gives_distinct_strings() {
        let bump = Bump::new();
        let mut interner = Interner::new(&bump);
        let foo: Interned<'_, FakeTag> = interner.from_str("foo");
        let bar: Interned<'_, FakeTag> = interner.from_str("bar");
        assert_eq!(foo.as_str(), "foo");
        assert_eq!(bar.as_str(), "bar");
    }

    #[test]
    fn interner_gives_same_obj() {
        let bump = Bump::new();
        let mut interner = Interner::new(&bump);
        let foo1: Interned<'_, FakeTag> = interner.from_str("foo");
        let foo2: Interned<'_, FakeTag> = interner.from_str("foo");
        assert_eq!(foo1, foo2);
        assert_eq!(foo2.as_str(), "foo");
    }
}
