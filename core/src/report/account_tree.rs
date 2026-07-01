//! Hierarchical view over a flat [`Balance`].
//!
//! Account names use `:` as a hierarchy separator (e.g. `Assets:Bank:Checking`).
//! [`AccountTree`] groups a [`Balance`] by those path segments and rolls the
//! amounts up to every ancestor, so callers can render tree-style reports with
//! per-level subtotals and a grand total.

use std::collections::BTreeMap;

use super::{balance::Balance, eval::Amount};

/// Tree of account path segments built from a [`Balance`],
/// where each node carries the rolled-up amount of its subtree.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AccountTree<'ctx> {
    root: Node<'ctx>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Node<'ctx> {
    children: BTreeMap<&'ctx str, Node<'ctx>>,
    /// Amount recorded directly on this account,
    /// `None` if the account itself never appeared in the balance.
    own: Option<Amount<'ctx>>,
    /// Sum of `own` and all descendants' amounts.
    total: Amount<'ctx>,
}

impl<'ctx> Node<'ctx> {
    fn compute_total(&mut self) {
        let mut total = self.own.clone().unwrap_or_default();
        for child in self.children.values_mut() {
            child.compute_total();
            total += &child.total;
        }
        // Keep the same normalization as [`Balance`], so a fully cancelled
        // subtree shows plain `0` instead of `0 JPY`.
        total.remove_zero_entries();
        self.total = total;
    }
}

impl<'ctx> AccountTree<'ctx> {
    /// Builds a tree from the given balance, splitting account names on `:`.
    pub fn from_balance(balance: &Balance<'ctx>) -> Self {
        let mut root = Node::default();
        for (account, amount) in balance.iter() {
            let mut node = &mut root;
            for segment in account.as_str().split(':') {
                node = node.children.entry(segment).or_default();
            }
            node.own = Some(amount.clone());
        }
        root.compute_total();
        Self { root }
    }

    /// Returns the top-level accounts, sorted by segment name.
    pub fn roots(&self) -> impl Iterator<Item = AccountNode<'_, 'ctx>> {
        self.root
            .children
            .iter()
            .map(|(segment, node)| AccountNode { segment, node })
    }

    /// Returns the grand total over all accounts.
    pub fn total(&self) -> &Amount<'ctx> {
        &self.root.total
    }
}

/// One account level inside an [`AccountTree`].
#[derive(Debug, Clone, Copy)]
pub struct AccountNode<'a, 'ctx> {
    segment: &'ctx str,
    node: &'a Node<'ctx>,
}

impl<'a, 'ctx> AccountNode<'a, 'ctx> {
    /// Returns the last path segment of this account
    /// (e.g. `Bank` for `Assets:Bank`).
    pub fn segment(&self) -> &'ctx str {
        self.segment
    }

    /// Returns the amount recorded directly on this account,
    /// or `None` if the account only exists as a prefix of other accounts.
    pub fn own_amount(&self) -> Option<&'a Amount<'ctx>> {
        self.node.own.as_ref()
    }

    /// Returns the rolled-up amount of this account and all its descendants.
    pub fn total(&self) -> &'a Amount<'ctx> {
        &self.node.total
    }

    /// Returns the child accounts, sorted by segment name.
    pub fn children(&self) -> impl Iterator<Item = AccountNode<'a, 'ctx>> + use<'a, 'ctx> {
        self.node
            .children
            .iter()
            .map(|(segment, node)| AccountNode { segment, node })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::super::context::ReportContext;

    fn collect<'a, 'ctx>(
        nodes: impl Iterator<Item = AccountNode<'a, 'ctx>>,
    ) -> Vec<AccountNode<'a, 'ctx>> {
        nodes.collect()
    }

    #[test]
    fn from_balance_empty() {
        let tree = AccountTree::from_balance(&Balance::default());

        assert_eq!(collect(tree.roots()).len(), 0);
        assert_eq!(tree.total(), &Amount::zero());
    }

    #[test]
    fn from_balance_rolls_up_to_ancestors() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodities.ensure("USD");
        let balance = Balance::from_map(hashmap! {
            ctx.accounts.ensure("Assets:Bank:Checking") => Amount::from_value(usd, dec!(1000)),
            ctx.accounts.ensure("Assets:Bank:Savings") => Amount::from_value(usd, dec!(5000)),
            ctx.accounts.ensure("Assets:Cash") => Amount::from_value(usd, dec!(200)),
            ctx.accounts.ensure("Expenses:Food") => Amount::from_value(usd, dec!(150)),
        });

        let tree = AccountTree::from_balance(&balance);

        assert_eq!(tree.total(), &Amount::from_value(usd, dec!(6350)));

        let roots = collect(tree.roots());
        assert_eq!(
            roots.iter().map(|x| x.segment()).collect::<Vec<_>>(),
            vec!["Assets", "Expenses"]
        );

        let assets = &roots[0];
        assert_eq!(assets.own_amount(), None);
        assert_eq!(assets.total(), &Amount::from_value(usd, dec!(6200)));

        let assets_children = collect(assets.children());
        assert_eq!(
            assets_children
                .iter()
                .map(|x| x.segment())
                .collect::<Vec<_>>(),
            vec!["Bank", "Cash"]
        );
        assert_eq!(
            assets_children[0].total(),
            &Amount::from_value(usd, dec!(6000))
        );
        assert_eq!(
            assets_children[1].own_amount(),
            Some(&Amount::from_value(usd, dec!(200)))
        );
        assert_eq!(collect(assets_children[1].children()).len(), 0);
    }

    #[test]
    fn from_balance_keeps_own_amount_on_intermediate_account() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let balance = Balance::from_map(hashmap! {
            ctx.accounts.ensure("Expenses") => Amount::from_value(jpy, dec!(30)),
            ctx.accounts.ensure("Expenses:Food") => Amount::from_value(jpy, dec!(70)),
        });

        let tree = AccountTree::from_balance(&balance);

        let roots = collect(tree.roots());
        assert_eq!(roots.len(), 1);
        let expenses = &roots[0];
        assert_eq!(
            expenses.own_amount(),
            Some(&Amount::from_value(jpy, dec!(30)))
        );
        assert_eq!(expenses.total(), &Amount::from_value(jpy, dec!(100)));
        assert_eq!(tree.total(), &Amount::from_value(jpy, dec!(100)));
    }

    #[test]
    fn from_balance_sums_multiple_commodities() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let jpy = ctx.commodities.ensure("JPY");
        let chf = ctx.commodities.ensure("CHF");
        let balance = Balance::from_map(hashmap! {
            ctx.accounts.ensure("Assets:Bank") => Amount::from_value(jpy, dec!(1000)),
            ctx.accounts.ensure("Assets:Broker") => Amount::from_value(chf, dec!(25)),
        });

        let tree = AccountTree::from_balance(&balance);

        assert_eq!(
            tree.total(),
            &Amount::from_iter([(jpy, dec!(1000)), (chf, dec!(25))])
        );
    }
}
