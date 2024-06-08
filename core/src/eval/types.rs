//! Module `types` defines eval specific types.

mod str_intern;

/// `&str` for accounts, interned by `Interner`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Account<'arena>(str_intern::InternedStr<'arena>);

impl<'arena> str_intern::FromInterned<'arena> for Account<'arena> {
    fn from_interned(v: str_intern::InternedStr<'arena>) -> Self {
        Self(v)
    }
}

impl<'arena> Account<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// `&str` for commodity, interned by `Interner`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Commodity<'arena>(str_intern::InternedStr<'arena>);

impl<'arena> str_intern::FromInterned<'arena> for Commodity<'arena> {
    fn from_interned(v: str_intern::InternedStr<'arena>) -> Self {
        Self(v)
    }
}

impl<'arena> Commodity<'arena> {
    /// Returns the `&str`.
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// `Interner` for `Account`.
pub struct AccountStore<'arena> {
    accounts: str_intern::Interner<'arena, Account<'arena>>,
}

impl<'arena> AccountStore<'arena> {
    pub(super) fn new(allocator: &'arena bumpalo::Bump) -> Self {
        Self {
            accounts: str_intern::Interner::new(allocator),
        }
    }

    pub fn intern(&mut self, value: &str) -> Account<'arena> {
        self.accounts.intern(value)
    }

    pub fn iter(&'arena self) -> str_intern::Iter<'arena, Account<'arena>> {
        self.accounts.iter()
    }
}

/// `Interner` for `Commodity`.
pub type CommodityStore<'arena> = str_intern::Interner<'arena, Commodity<'arena>>;
