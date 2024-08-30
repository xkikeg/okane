//! Defines commodity and its related types.

use std::collections::HashMap;

use bumpalo::Bump;

use crate::repl::pretty_decimal::PrettyDecimal;

use super::intern::{FromInterned, InternError, InternStore, InternedStr};

/// `&str` for commodities, interned within the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Commodity<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for Commodity<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }

    fn as_interned(&self) -> InternedStr<'arena> {
        self.0
    }
}

impl<'arena> Commodity<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// `Interner` for `Commodity`.
pub(super) struct CommodityStore<'arena> {
    intern: InternStore<'arena, Commodity<'arena>>,
}

impl<'arena> CommodityStore<'arena> {
    /// Creates a new instance.
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            intern: InternStore::new(arena),
        }
    }

    /// Facade for [InternStore::ensure].
    pub fn ensure(&mut self, value: &str) -> Commodity<'arena> {
        self.intern.ensure(value)
    }

    /// Inserts given `value` as always canonical.
    /// Returns the registered canonical, or error if given `value` is already registered as alias.
    /// Facade for [InternStore::insert_canonical].
    pub fn insert_canonical(&mut self, value: &str) -> Result<Commodity<'arena>, InternError> {
        self.intern.insert_canonical(value)
    }

    /// Inserts given `value` as always alias of `canonical`.
    /// Returns error if given `value` is already registered as canonical.
    /// Facade for [InternStore::insert_alias].
    pub fn insert_alias(
        &mut self,
        value: &str,
        canonical: Commodity<'arena>,
    ) -> Result<(), InternError> {
        self.intern.insert_alias(value, canonical)
    }
}
