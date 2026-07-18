//! Defines account and its related types.

use bumpalo::Bump;
use bumpalo_intern::direct::{DirectInternStore, FromInterned, InternedStr, Iter, OccupiedError};

/// Interned `&str` for accounts within the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Account<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for Account<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }

    fn as_interned(&self) -> InternedStr<'arena> {
        self.0
    }
}

impl<'arena> Account<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// Manages [`Account`] instances.
pub struct AccountStore<'arena> {
    intern: DirectInternStore<'arena, Account<'arena>>,
}

impl<'arena> AccountStore<'arena> {
    /// Creates a new instance.
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            intern: DirectInternStore::new(arena),
        }
    }

    /// Returns the Account with the given `value`,
    /// potentially resolving the alias.
    /// If not available, registers the given `value` as the canonical.
    pub fn ensure(&mut self, value: &str) -> Account<'arena> {
        self.intern.ensure(value)
    }

    /// Returns the Account with the given `value` if and only if it's already registered.
    pub fn resolve(&self, value: &str) -> Option<Account<'arena>> {
        self.intern.resolve(value)
    }

    /// Registers given `value` as always alias of `canonical`.
    /// Returns error if given `value` is already registered as canonical.
    pub fn register_alias(
        &mut self,
        value: &str,
        canonical: Account<'arena>,
    ) -> Result<(), OccupiedError<'arena, Account<'arena>>> {
        self.intern.register_alias(value, canonical)
    }

    /// Returns the Iterator for all elements.
    pub fn iter(&self) -> Iter<'arena, '_, Account<'arena>> {
        self.intern.iter()
    }
}
