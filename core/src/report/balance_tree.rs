//! Defines the [`BalanceTree`], tree of balance.

use std::collections::{HashMap, HashSet};

use super::account::{Account, AccountAggregate, AccountTreeKey};
use super::balance::Balance;
use super::context::ReportContext;
use super::eval::Amount;
use super::query::QueryError;

/// Accumulated balance, considering the account hierarchy.
#[derive(Debug)]
pub struct BalanceTree<'ctx> {
    /// Amount labeled with AccountAggregate.
    /// Sorted alphabetically, also stored in hierarchical.
    values: Vec<BalanceTreeNode<'ctx>>,
}

impl<'ctx> BalanceTree<'ctx> {
    pub fn create(ctx: &ReportContext<'ctx>, balance: Balance<'ctx>) -> Result<Self, QueryError> {
        let mut values: Vec<BalanceTreeNode<'ctx>> = vec![BalanceTreeNode::placeholder(
            AccountTreeKey::Root,
            0,
            Amount::zero(),
        )];
        // To tell if the account aggregate is required or not,
        // we need to maintain `required` to avoid going into unnecessary branch in accumulate DFS.
        let mut required: HashSet<AccountAggregate<'ctx>> = HashSet::new();
        for (account, _) in balance.iter() {
            let mut account = (*account).into();
            required.insert(account);
            while let Some(parent) = parent_of(ctx, account)? {
                if !required.insert(parent) {
                    // other already inserted, no need to do.
                    break;
                }
                account = parent;
            }
        }
        accumulate(&mut balance.into_map(), &mut values, ctx, &required, 0)?;
        Ok(Self { values })
    }

    /// Returns all nodes in the flat vector.
    ///
    /// The order is both alphabetical (by account name) and a pre-order
    /// (DFS) flattening of the account hierarchy; index 0 is always the
    /// synthetic [`AccountTreeKey::Root`].
    pub fn nodes(&self) -> &[BalanceTreeNode<'ctx>] {
        &self.values
    }

    /// Consumes the tree and returns the owned node vector.
    /// See [`Self::nodes`] for the ordering guarantees.
    pub fn into_nodes(self) -> Vec<BalanceTreeNode<'ctx>> {
        self.values
    }
}

fn parent_of<'ctx>(
    ctx: &ReportContext<'ctx>,
    account: AccountAggregate<'ctx>,
) -> Result<Option<AccountAggregate<'ctx>>, QueryError> {
    let parent = ctx.account_tree.parent(account).ok_or_else(|| {
        QueryError::Internal(format!(
            "given account {} was not properly registered in the AccountTree",
            account.as_str()
        ))
    })?;
    Ok(match parent {
        AccountTreeKey::Root => None,
        AccountTreeKey::Descendant(parent) => Some(parent),
    })
}

fn label(account: AccountTreeKey<'_>) -> &str {
    account
        .as_aggregate()
        .map(|x| x.as_str())
        .unwrap_or_default()
}

fn accumulate<'ctx>(
    balance: &mut HashMap<Account<'ctx>, Amount<'ctx>>,
    values: &mut Vec<BalanceTreeNode<'ctx>>,
    ctx: &ReportContext<'ctx>,
    required: &HashSet<AccountAggregate<'ctx>>,
    i: usize,
) -> Result<(Amount<'ctx>, NodeRange), QueryError> {
    if i >= values.len() {
        return Err(QueryError::Internal(format!(
            "BalanceTree accumulate is trying to access values[{i}] for size {}",
            values.len(),
        )));
    }
    let account = values[i].account;
    let depth = values[i].depth;
    let mut total = values[i].self_amount.clone();
    let mut children: Vec<NodeRange> = Vec::new();
    for account in ctx.account_tree.children(account).ok_or_else(|| {
        QueryError::Internal(format!(
            "account {} isn't registered in the AccountTree",
            label(account)
        ))
    })? {
        if !required.contains(account) {
            continue;
        }
        let j = values.len();
        let amount = match account {
            AccountAggregate::Account(account) => balance.remove(account).unwrap_or_default(),
            AccountAggregate::Ancestor(_) => Amount::default(),
        };
        values.push(BalanceTreeNode::placeholder(
            (*account).into(),
            depth + 1,
            amount,
        ));
        let (child_amount, child_range) = accumulate(balance, values, ctx, required, j)?;
        total += child_amount;
        children.push(child_range);
    }
    let cover = NodeRange::cover(i, &children)?;
    total.remove_zero_entries();
    values[i].subtree_amount = total.clone();
    values[i].children = children;
    values[i].range = cover;
    Ok((total, cover))
}

/// Node of the each element.
#[derive(Debug, PartialEq, Eq)]
pub struct BalanceTreeNode<'ctx> {
    /// Account key of this node ([`AccountTreeKey::Root`] for the synthetic root).
    pub account: AccountTreeKey<'ctx>,
    /// Depth in the tree: root is `0`, its children `1`, and so on.
    pub depth: u16,
    /// Amount of the node itself, excluding descendants.
    pub self_amount: Amount<'ctx>,
    /// Amount of the node and all its descendants.
    pub subtree_amount: Amount<'ctx>,
    /// Children, expressed as Vector index range.
    children: Vec<NodeRange>,
    /// Contiguous span of this node plus all its descendants in the flat vector.
    range: NodeRange,
}

impl<'ctx> BalanceTreeNode<'ctx> {
    /// Creates a new instance with place holder.
    fn placeholder(account: AccountTreeKey<'ctx>, depth: u16, self_amount: Amount<'ctx>) -> Self {
        Self {
            account,
            depth,
            self_amount,
            // all placeholders below
            subtree_amount: Amount::zero(),
            children: Vec::new(),
            range: NodeRange { start: 0, size: 0 },
        }
    }

    /// Returns `true` when the node has at least one child.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Range of this node and all its descendants in [`BalanceTree::nodes`].
    ///
    /// The node itself is at `range.start`; its descendants (if any) occupy
    /// `range.start + 1 .. range.end`.
    pub fn subtree_range(&self) -> std::ops::Range<usize> {
        self.range.into()
    }
}

/// Range with Copy dedicated for node range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeRange {
    pub start: usize,
    pub size: usize,
}

impl NodeRange {
    /// Constructs a single NodeRange as a cover of given `[i]` + `ranges`.
    /// Each range must be in order (smaller to larger), continuous.
    fn cover(i: usize, ranges: &[NodeRange]) -> Result<Self, QueryError> {
        let start = i;
        let mut size = 1;
        for range in ranges {
            if start + size != range.start {
                return Err(QueryError::Internal(format!(
                    "NodeRange::cover can only cover continuous and mutually exclusive ranges: total start={start}, size={size}, current start={}",
                    range.start
                )));
            }
            size += range.size;
        }
        Ok(Self { start, size })
    }
}

impl From<std::ops::Range<usize>> for NodeRange {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start: value.start,
            size: value.end.checked_sub(value.start).unwrap(),
        }
    }
}

impl From<NodeRange> for std::ops::Range<usize> {
    fn from(value: NodeRange) -> Self {
        Self {
            start: value.start,
            end: value.start + value.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn simple_tree_for_non_overlapping_balance() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let chf = ctx.commodities.ensure("CHF");
        let jpy = ctx.commodities.ensure("JPY");
        let usd = ctx.commodities.ensure("USD");

        let balance = Balance::from_iter(hashmap! {
            ctx.accounts.ensure("Assets") =>
                Amount::from_iter([(chf, dec!(10)), (usd, dec!(1))]),
            ctx.accounts.ensure("Expenses") =>
                Amount::from_value(jpy, Decimal::ZERO),
        });
        ctx.account_tree.construct(&ctx.accounts);

        let tree = BalanceTree::create(&ctx, balance).unwrap();
        let got = tree.into_nodes();
        let want = vec![
            BalanceTreeNode {
                account: AccountTreeKey::Root,
                depth: 0,
                self_amount: Amount::zero(),
                subtree_amount: Amount::from_iter([
                    (chf, dec!(10)),
                    (usd, dec!(1)),
                ]),
                children: vec![(1..2).into(), (2..3).into()],
                range: (0..3).into(),
            },
            BalanceTreeNode {
                account: ctx.accounts.resolve("Assets").unwrap().into(),
                depth: 1,
                self_amount: Amount::from_iter([(chf, dec!(10)), (usd, dec!(1))]),
                subtree_amount: Amount::from_iter([(chf, dec!(10)), (usd, dec!(1))]),
                children: vec![],
                range: (1..2).into(),
            },
            BalanceTreeNode {
                account: ctx.accounts.resolve("Expenses").unwrap().into(),
                depth: 1,
                self_amount: Amount::zero(),
                subtree_amount: Amount::zero(),
                children: vec![],
                range: (2..3).into(),
            },
        ];
        assert_eq!(want, got);
    }

    /// Resolves `name` to its tree key, whether it is a posted account or an
    /// ancestor-only prefix.
    fn account_key<'a>(ctx: &ReportContext<'a>, name: &str) -> AccountTreeKey<'a> {
        if let Some(account) = ctx.accounts.resolve(name) {
            account.into()
        } else {
            ctx.account_tree
                .resolve_prefix(name)
                .expect("account or prefix")
                .into()
        }
    }

    /// A hierarchy exercising the tricky cases: a posted account that is also a
    /// parent (`Assets:Bank`); ancestor-only nodes (`Assets`, `Expenses`,
    /// `Expenses:Travel`); a name that is a prefix of a sibling but not its
    /// parent (`Expenses:Property` vs `Expenses:Property Tax`); and a child that
    /// sorts *after* such a sibling in raw ASCII (`Expenses:Property:Maintenance`
    /// vs `Expenses:Property Tax`, since `':'` > `' '`) yet must stay grouped
    /// under its parent — i.e. the tree order is hierarchical, not a flat sort.
    #[test]
    fn tree_with_overlapping_and_ancestor_accounts() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodities.ensure("USD");

        let balance = Balance::from_iter(hashmap! {
            ctx.accounts.ensure("Assets:Bank") => Amount::from_value(usd, dec!(1)),
            ctx.accounts.ensure("Assets:Bank:Checking") => Amount::from_value(usd, dec!(3)),
            ctx.accounts.ensure("Assets:Cash") => Amount::from_value(usd, dec!(5)),
            ctx.accounts.ensure("Expenses:Property") => Amount::from_value(usd, dec!(1_000)),
            ctx.accounts.ensure("Expenses:Property:Maintenance") => Amount::from_value(usd, dec!(23)),
            ctx.accounts.ensure("Expenses:Property Tax") => Amount::from_value(usd, dec!(13)),
            ctx.accounts.ensure("Expenses:Travel:Taxi") => Amount::from_value(usd, dec!(11)),
            // want to see if Expenses:Travel gets Amount::zero properly .
            ctx.accounts.ensure("Expenses:Travel:Train") => Amount::from_value(usd, dec!(-11)),
        });
        ctx.account_tree.construct(&ctx.accounts);

        let amount = |v| Amount::from_value(usd, v);
        let key = |name: &str| account_key(&ctx, name);

        let tree = BalanceTree::create(&ctx, balance).unwrap();
        let got = tree.into_nodes();
        let want = vec![
            // 0: Root — spans the whole vector.
            BalanceTreeNode {
                account: AccountTreeKey::Root,
                depth: 0,
                self_amount: Amount::zero(),
                subtree_amount: amount(dec!(1045)),
                children: vec![(1..5).into(), (5..12).into()],
                range: (0..12).into(),
            },
            // 1: Assets — ancestor with no posting of its own.
            BalanceTreeNode {
                account: key("Assets"),
                depth: 1,
                self_amount: Amount::zero(),
                subtree_amount: amount(dec!(9)),
                children: vec![(2..4).into(), (4..5).into()],
                range: (1..5).into(),
            },
            // 2: Assets:Bank — posted *and* a parent; keeps its own 1 USD.
            BalanceTreeNode {
                account: key("Assets:Bank"),
                depth: 2,
                self_amount: amount(dec!(1)),
                subtree_amount: amount(dec!(4)),
                children: vec![(3..4).into()],
                range: (2..4).into(),
            },
            // 3: Assets:Bank:Checking — leaf.
            BalanceTreeNode {
                account: key("Assets:Bank:Checking"),
                depth: 3,
                self_amount: amount(dec!(3)),
                subtree_amount: amount(dec!(3)),
                children: vec![],
                range: (3..4).into(),
            },
            // 4: Assets:Cash — leaf sibling of Assets:Bank.
            BalanceTreeNode {
                account: key("Assets:Cash"),
                depth: 2,
                self_amount: amount(dec!(5)),
                subtree_amount: amount(dec!(5)),
                children: vec![],
                range: (4..5).into(),
            },
            // 5: Expenses — ancestor.
            BalanceTreeNode {
                account: key("Expenses"),
                depth: 1,
                self_amount: Amount::zero(),
                subtree_amount: amount(dec!(1036)),
                children: vec![(6..8).into(), (8..9).into(), (9..12).into()],
                range: (5..12).into(),
            },
            // 6: Expenses:Property — posted and the parent of Maintenance.
            BalanceTreeNode {
                account: key("Expenses:Property"),
                depth: 2,
                self_amount: amount(dec!(1_000)),
                subtree_amount: amount(dec!(1_023)),
                children: vec![(7..8).into()],
                range: (6..8).into(),
            },
            // 7: Expenses:Property:Maintenance — child of Property; stays grouped
            // under it even though "Property Tax" sorts before it by full name.
            BalanceTreeNode {
                account: key("Expenses:Property:Maintenance"),
                depth: 3,
                self_amount: amount(dec!(23)),
                subtree_amount: amount(dec!(23)),
                children: vec![],
                range: (7..8).into(),
            },
            // 8: Expenses:Property Tax — sibling leaf of Property (not its child).
            BalanceTreeNode {
                account: key("Expenses:Property Tax"),
                depth: 2,
                self_amount: amount(dec!(13)),
                subtree_amount: amount(dec!(13)),
                children: vec![],
                range: (8..9).into(),
            },
            // 9: Expenses:Travel — ancestor with a single posted descendant.
            BalanceTreeNode {
                account: key("Expenses:Travel"),
                depth: 2,
                self_amount: Amount::zero(),
                subtree_amount: Amount::zero(),
                children: vec![(10..11).into(), (11..12).into()],
                range: (9..12).into(),
            },
            // 10: Expenses:Travel:Taxi — leaf.
            BalanceTreeNode {
                account: key("Expenses:Travel:Taxi"),
                depth: 3,
                self_amount: amount(dec!(11)),
                subtree_amount: amount(dec!(11)),
                children: vec![],
                range: (10..11).into(),
            },
            // 11: Expenses:Travel:Train — leaf.
            BalanceTreeNode {
                account: key("Expenses:Travel:Train"),
                depth: 3,
                self_amount: amount(dec!(-11)),
                subtree_amount: amount(dec!(-11)),
                children: vec![],
                range: (11..12).into(),
            },
        ];
        assert_eq!(want, got);
    }
}
