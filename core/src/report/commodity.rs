//! Defines commodity and its related types.

use std::{collections::HashMap, fmt::Display};

use bumpalo::Bump;

use crate::syntax::pretty_decimal::PrettyDecimal;

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

impl Display for Commodity<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<'arena> Commodity<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// Owned [`Commodity`], which is just [`String`].
/// Useful to store in the error.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct OwnedCommodity(String);

impl OwnedCommodity {
    /// Creates a new [`OwnedCommodity`] instance.
    pub fn from_string(v: String) -> Self {
        Self(v)
    }

    /// Returns the underlying [`&str`].
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the underlying [`String`].
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<Commodity<'_>> for OwnedCommodity {
    fn from(value: Commodity<'_>) -> Self {
        Self(value.as_str().to_string())
    }
}

impl Display for OwnedCommodity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// `Interner` for [`Commodity`].
pub(super) struct CommodityStore<'arena> {
    intern: InternStore<'arena, Commodity<'arena>>,
    formatting: HashMap<Commodity<'arena>, PrettyDecimal>,
}

impl<'arena> CommodityStore<'arena> {
    /// Creates a new instance.
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            intern: InternStore::new(arena),
            formatting: HashMap::new(),
        }
    }

    /// Returns the Commodity with the given `value`,
    /// potentially resolving the alias.
    /// If not available, registers the given `value` as the canonical.
    pub fn ensure(&mut self, value: &str) -> Commodity<'arena> {
        self.intern.ensure(value)
    }

    /// Returns the Commodity with the given `value` if and only if it's already registered.
    pub fn resolve(&self, value: &str) -> Option<Commodity<'arena>> {
        self.intern.resolve(value)
    }

    /// Inserts given `value` as always canonical.
    /// Returns the registered canonical, or error if given `value` is already registered as alias.
    /// Facade for [InternStore::insert_canonical].
    pub(super) fn insert_canonical(
        &mut self,
        value: &str,
    ) -> Result<Commodity<'arena>, InternError> {
        self.intern.insert_canonical(value)
    }

    /// Inserts given `value` as always alias of `canonical`.
    /// Returns error if given `value` is already registered as canonical.
    /// Facade for [InternStore::insert_alias].
    pub(super) fn insert_alias(
        &mut self,
        value: &str,
        canonical: Commodity<'arena>,
    ) -> Result<(), InternError> {
        self.intern.insert_alias(value, canonical)
    }

    pub fn get_decimal_point(&self, commodity: Commodity<'arena>) -> Option<u32> {
        self.formatting.get(&commodity).map(|x| x.value.scale())
    }

    pub fn set_format(&mut self, commodity: Commodity<'arena>, format: PrettyDecimal) {
        self.formatting.insert(commodity, format);
    }
}
