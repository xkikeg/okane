//! Defines commodity and its related types.

use std::borrow::Cow;
use std::fmt::Display;

use bumpalo::Bump;
use bumpalo_intern::dense::{DenseInternStore, InternTag, Interned, Keyed, OccupiedError};
use pretty_decimal::PrettyDecimal;

/// `&str` for commodities, interned within the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Commodity<'arena>(&'arena str);

impl<'a> Keyed<'a> for Commodity<'a> {
    fn intern_key(&self) -> &'a str {
        self.0
    }
}
impl<'a> Interned<'a> for Commodity<'a> {
    type View<'b> = Commodity<'b>;

    fn intern_from<'b>(arena: &'a Bump, view: Self::View<'b>) -> (&'a str, Self) {
        let key = arena.alloc_str(view.0);
        (key, Commodity(key))
    }
}

impl Display for Commodity<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<'a> Commodity<'a> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'a str {
        self.0
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct CommodityTag<'a>(InternTag<Commodity<'a>>);

impl<'ctx> CommodityTag<'ctx> {
    /// Returns the index of the commodity.
    /// Note this index is dense, so you can assume it fits in 0..len range.
    pub fn as_index(&self) -> usize {
        self.0.as_index()
    }

    /// Takes the str if possible.
    pub(super) fn to_str_lossy(self, commodity_store: &CommodityStore<'ctx>) -> Cow<'ctx, str> {
        match commodity_store.get(self) {
            Some(x) => Cow::Borrowed(x.as_str()),
            None => Cow::Owned(format!("unknown#{}", self.as_index())),
        }
    }

    /// Converts the self into [`OwnedCommodity`].
    /// If the tag isn't registered in the `commodity_store`,
    /// it'll print "unknown#xx" as the place holder.
    pub(super) fn to_owned_lossy(self, commodity_store: &CommodityStore<'ctx>) -> OwnedCommodity {
        OwnedCommodity::from_string(self.to_str_lossy(commodity_store).into_owned())
    }
}

/// Interner for [`Commodity`].
pub(super) struct CommodityStore<'arena> {
    intern: DenseInternStore<'arena, Commodity<'arena>>,
    formatting: CommodityMap<PrettyDecimal>,
}

impl<'arena> std::fmt::Debug for CommodityStore<'arena> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommodityStore")
            .field("intern", &format!("[{} commodities]", self.intern.len()))
            .finish()
    }
}

impl<'arena> CommodityStore<'arena> {
    /// Creates a new instance.
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            intern: DenseInternStore::new(arena),
            formatting: CommodityMap::new(),
        }
    }

    /// Returns the Commodity with the given `value`,
    /// potentially resolving the alias.
    /// If not available, registers the given `value` as the canonical.
    pub fn ensure(&mut self, value: &'_ str) -> CommodityTag<'arena> {
        CommodityTag(self.intern.ensure(Commodity(value)))
    }

    /// Returns the tag corresponding [`Commodity`].
    pub fn get(&self, tag: CommodityTag<'arena>) -> Option<Commodity<'arena>> {
        self.intern.get(tag.0)
    }

    /// Returns the Commodity with the given `value` if and only if it's already registered.
    pub fn resolve(&self, value: &str) -> Option<CommodityTag<'arena>> {
        self.intern.resolve(value).map(CommodityTag)
    }

    #[cfg(test)]
    pub fn insert(
        &mut self,
        value: &str,
    ) -> Result<CommodityTag<'arena>, OccupiedError<Commodity<'arena>>> {
        self.intern.try_insert(Commodity(value)).map(CommodityTag)
    }

    /// Inserts given `value` as always alias of `canonical`.
    /// Returns error if given `value` is already registered as canonical.
    /// Facade for [InternStore::insert_alias].
    pub(super) fn insert_alias(
        &mut self,
        value: &str,
        canonical: CommodityTag<'arena>,
    ) -> Result<(), OccupiedError<Commodity<'arena>>> {
        self.intern.insert_alias(Commodity(value), canonical.0)
    }

    /// Returns the precision of the `commodity` if specified.
    #[inline]
    pub(super) fn get_decimal_point(&self, commodity: CommodityTag<'arena>) -> Option<u32> {
        match self.formatting.get(commodity) {
            Some(x) => Some(x.scale()),
            _ => None,
        }
    }

    /// Sets the format of the `commodity` as [`PrettyDecimal`].
    #[inline]
    pub(super) fn set_format(&mut self, commodity: CommodityTag<'arena>, format: PrettyDecimal) {
        self.formatting.set(commodity, format);
    }

    /// Returns the total length of the commodity.
    #[inline]
    pub fn len(&self) -> usize {
        self.intern.len()
    }
}

/// Map from CommodityTag<'arena> to value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommodityMap<T> {
    inner: Vec<Option<T>>,
}

impl<T> CommodityMap<T> {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates a new instance with a given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Returns the reference to the corresponding element.
    pub fn get(&self, k: CommodityTag<'_>) -> Option<&T> {
        match self.inner.get(k.as_index()) {
            Some(Some(r)) => Some(r),
            Some(None) | None => None,
        }
    }
}

impl<T: Clone> CommodityMap<T> {
    /// Returns the mutable reference corresponding to the given `k`.
    pub fn get_mut(&mut self, k: CommodityTag<'_>) -> &mut Option<T> {
        self.ensure_size(k);
        &mut self.inner[k.as_index()]
    }

    /// Sets the given key value.
    pub fn set(&mut self, k: CommodityTag<'_>, v: T) {
        self.ensure_size(k);
        self.inner[k.as_index()] = Some(v);
    }

    /// Ensure size for given `k`.
    #[inline]
    fn ensure_size(&mut self, k: CommodityTag<'_>) {
        if self.inner.len() <= k.as_index() {
            self.inner.resize(k.as_index() + 1, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn to_owned_lossy() {
        let arena = Bump::new();
        let mut commodities = CommodityStore::new(&arena);
        let chf = commodities.insert("CHF").unwrap();

        assert_eq!(
            OwnedCommodity::from_string("CHF".to_string()),
            chf.to_owned_lossy(&commodities)
        );

        let unknown = CommodityTag(InternTag::new(1));

        assert_eq!(
            OwnedCommodity::from_string("unknown#1".to_string()),
            unknown.to_owned_lossy(&commodities)
        );
    }

    #[test]
    fn get_decimal_point_returns_none_if_unspecified() {
        let arena = Bump::new();
        let mut commodities = CommodityStore::new(&arena);
        let jpy = commodities.insert("JPY").unwrap();

        assert_eq!(None, commodities.get_decimal_point(jpy));
    }

    #[test]
    fn get_decimal_point_returns_some_if_set() {
        let arena = Bump::new();
        let mut commodities = CommodityStore::new(&arena);
        let jpy = commodities.insert("JPY").unwrap();
        commodities.set_format(jpy, PrettyDecimal::comma3dot(dec!(1.234)));

        assert_eq!(Some(3), commodities.get_decimal_point(jpy));
    }
}
