//! Defines account and its related types.

use std::collections::{HashMap, hash_map};

use bumpalo::Bump;
use bumpalo_intern::direct::{
    DirectInternStore, FromInterned, InternedStr, OccupiedError, StoredValue,
};

/// Interned `&str` for account with the `'arena` bounded allocator lifetime.
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
    #[inline]
    pub fn as_str(&self) -> &'arena str {
        self.0.as_str()
    }
}

/// Manages [`Account`] instances.
pub struct AccountStore<'arena> {
    /// Interned Account store.
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

    /// Returns the [`Iterator`] for just `Account`.
    /// Order is unspecified.
    pub fn iter(&self) -> impl Iterator<Item = Account<'arena>> {
        self.intern.iter().filter_map(|v| match v {
            StoredValue::Alias { .. } => None,
            StoredValue::Canonical(a) => Some(a),
        })
    }
}

/// Interned `&str` for account prefixes with the `'arena` bounded allocator lifetime.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AccountPrefix<'arena>(InternedStr<'arena>);

impl<'arena> FromInterned<'arena> for AccountPrefix<'arena> {
    fn from_interned(v: InternedStr<'arena>) -> Self {
        Self(v)
    }

    fn as_interned(&self) -> InternedStr<'arena> {
        self.0
    }
}

/// Item returned by [`AccountStore::parents()`].
/// Key of the [`BalanceTree`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AccountAggregate<'arena> {
    /// Concrete account which appeared in the Ledger file.
    Account(Account<'arena>),
    /// Ancestor of the tree acounts, which doesn't exist in the Ledger.
    Ancestor(AccountPrefix<'arena>),
}

impl<'arena> From<Account<'arena>> for AccountAggregate<'arena> {
    fn from(value: Account<'arena>) -> Self {
        Self::Account(value)
    }
}

impl<'arena> From<AccountPrefix<'arena>> for AccountAggregate<'arena> {
    fn from(value: AccountPrefix<'arena>) -> Self {
        Self::Ancestor(value)
    }
}

impl<'arena> AccountAggregate<'arena> {
    #[inline]
    pub fn as_str(&self) -> &'arena str {
        match self {
            AccountAggregate::Account(Account(v)) => v.as_str(),
            AccountAggregate::Ancestor(v) => v.as_interned().as_str(),
        }
    }
}

/// Represents the tree structure of the accounts, stored in the [`AccountStore`].
pub struct AccountTree<'arena> {
    prefixes: DirectInternStore<'arena, AccountPrefix<'arena>>,
    root_children: Vec<AccountAggregate<'arena>>,
    nodes: HashMap<AccountAggregate<'arena>, Node<'arena>>,
}

impl<'arena> AccountTree<'arena> {
    /// Creates a new instance.
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            prefixes: DirectInternStore::new(&arena),
            root_children: Vec::new(),
            nodes: HashMap::new(),
        }
    }

    /// Constructs the tree state from the given [`AccountStore`].
    pub fn construct(&mut self, accounts: &AccountStore<'arena>) {
        self.root_children.clear();
        self.nodes.clear();
        // need to copy first to avoid mutation rule.
        for account in accounts.iter() {
            if self.nodes.contains_key(&account.into()) {
                continue;
            }
            let mut next = Some(AccountAggregate::Account(account));
            while let Some(current) = next {
                next = match self.ensure_parent(accounts, current) {
                    None => {
                        self.nodes.entry(current).or_insert_with(Node::placeholder);
                        self.root_children.push(current);
                        None
                    }
                    Some(parent) => {
                        let cn = self.nodes.entry(current).or_insert_with(Node::placeholder);
                        cn.parent = AccountTreeKey::Descendant(parent);
                        let (children, candidate) = match self.nodes.entry(parent) {
                            hash_map::Entry::Occupied(existing) => {
                                (&mut existing.into_mut().children, None)
                            }
                            hash_map::Entry::Vacant(new) => {
                                (&mut new.insert(Node::placeholder()).children, Some(parent))
                            }
                        };
                        children.push(current);
                        candidate
                    }
                };
            }
        }
        self.root_children.sort_by_key(AccountAggregate::as_str);
        for node in self.nodes.values_mut() {
            node.children.sort_by_key(AccountAggregate::as_str);
        }
    }

    /// Internal method to ensure parent of given node.
    fn ensure_parent(
        &mut self,
        accounts: &AccountStore<'arena>,
        child: AccountAggregate<'arena>,
    ) -> Option<AccountAggregate<'arena>> {
        // if this is `None`, this means the given child is simple "Foo", and the parent is the root.
        let (parent, _) = child.as_str().rsplit_once(':')?;
        Some(match accounts.resolve(parent) {
            Some(v) => AccountAggregate::Account(v),
            None => AccountAggregate::Ancestor(self.prefixes.ensure(parent)),
        })
    }

    /// Resolves a prefix corresponding to the given label.
    /// Note if the given label is the account, this still returns `None`.
    /// This won't be realistically problem as caller should have [`AccountStore`] as well.
    pub fn resolve_prefix(&self, label: &str) -> Option<AccountPrefix<'arena>> {
        self.prefixes.resolve(label)
    }

    /// Returns the parent of the given node.
    /// `None` if the given `account` is not found in the tree.
    pub fn parent<T: Into<AccountAggregate<'arena>>>(
        &self,
        account: T,
    ) -> Option<AccountTreeKey<'arena>> {
        self.nodes.get(&account.into()).map(|node| node.parent)
    }

    /// Returns the children of the given node.
    /// it returns empty for unknown or leaf.
    pub fn children<T: Into<AccountTreeKey<'arena>>>(
        &self,
        key: T,
    ) -> Option<&[AccountAggregate<'arena>]> {
        match key.into() {
            AccountTreeKey::Root => Some(&self.root_children),
            AccountTreeKey::Descendant(account) => {
                self.nodes.get(&account).map(|n| n.children.as_slice())
            }
        }
    }
}

/// Key used for [`AccountTree`] to query.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AccountTreeKey<'arena> {
    /// Root of the tree.
    Root,
    Descendant(AccountAggregate<'arena>),
}

impl<'arena> AccountTreeKey<'arena> {
    /// Returns as [`Option<AccountAggregate>`].
    /// It will be `Some` unless the self is root.
    #[inline]
    pub fn as_aggregate(self) -> Option<AccountAggregate<'arena>> {
        match self {
            Self::Root => None,
            Self::Descendant(a) => Some(a),
        }
    }
}

impl<'arena> From<Account<'arena>> for AccountTreeKey<'arena> {
    fn from(value: Account<'arena>) -> Self {
        Self::Descendant(value.into())
    }
}

impl<'arena> From<AccountPrefix<'arena>> for AccountTreeKey<'arena> {
    fn from(value: AccountPrefix<'arena>) -> Self {
        Self::Descendant(value.into())
    }
}

impl<'arena> From<AccountAggregate<'arena>> for AccountTreeKey<'arena> {
    fn from(value: AccountAggregate<'arena>) -> Self {
        Self::Descendant(value)
    }
}

/// Node of the [`AccountTree`].
#[derive(Debug, Clone)]
struct Node<'arena> {
    parent: AccountTreeKey<'arena>,
    children: Vec<AccountAggregate<'arena>>,
}

impl<'arena> Node<'arena> {
    /// Creates a place holder.
    fn placeholder() -> Self {
        Self {
            parent: AccountTreeKey::Root,
            children: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    mod account_tree {
        use super::assert_eq;
        use super::*;

        #[test]
        fn init_reuses_previous_value() {
            let arena = Bump::new();
            let mut accounts = AccountStore::new(&arena);
            let mut tree = AccountTree::new(&arena);

            let foo = accounts.ensure("Assets:Foo");
            tree.construct(&accounts);
            let assets = tree.parent(AccountAggregate::Account(foo)).unwrap();

            tree.construct(&accounts);
            let assets2 = tree.parent(AccountAggregate::Account(foo)).unwrap();

            assert_eq!(assets, assets2);
        }

        #[test]
        fn empty_tree() {
            let arena = Bump::new();
            let accounts = AccountStore::new(&arena);
            let mut tree = AccountTree::new(&arena);

            tree.construct(&accounts);

            assert!(tree.children(AccountTreeKey::Root).unwrap().is_empty());
        }

        #[test]
        fn tree_structure() {
            let arena = Bump::new();
            let mut accounts = AccountStore::new(&arena);

            let foo = accounts.ensure("Assets:Banks:Foo");
            let bar = accounts.ensure("Assets:Banks:Bar");
            let food = accounts.ensure("Expenses:Food");
            let food_special = accounts.ensure("Expenses:Food:Special");

            let mut tree = AccountTree::new(&arena);
            tree.construct(&accounts);

            let assets = tree.resolve_prefix("Assets").unwrap();
            let banks = tree.resolve_prefix("Assets:Banks").unwrap();
            let expenses = tree.resolve_prefix("Expenses").unwrap();
            let parent_of: [(AccountTreeKey, AccountAggregate); _] = [
                (AccountTreeKey::Root, assets.into()),
                (AccountTreeKey::Root, expenses.into()),
                (assets.into(), banks.into()),
                (banks.into(), foo.into()),
                (banks.into(), bar.into()),
                (expenses.into(), food.into()),
                (food.into(), food_special.into()),
            ];
            for (expected, child) in parent_of {
                assert_eq!(expected, tree.parent(child).unwrap());
            }
            let children_of: [(&[AccountAggregate], AccountTreeKey); _] = [
                (
                    &[
                        AccountAggregate::Ancestor(assets),
                        AccountAggregate::Ancestor(expenses),
                    ],
                    AccountTreeKey::Root,
                ),
                (&[AccountAggregate::Ancestor(banks)], assets.into()),
                (
                    // alphabetically sorted.
                    &[
                        AccountAggregate::Account(bar),
                        AccountAggregate::Account(foo),
                    ],
                    banks.into(),
                ),
                (&[AccountAggregate::Account(food)], expenses.into()),
                (&[AccountAggregate::Account(food_special)], food.into()),
            ];
            for (expected, parent) in children_of {
                assert_eq!(expected, tree.children(parent).unwrap());
            }
        }
    }
}
