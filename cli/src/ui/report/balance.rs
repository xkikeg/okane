//! Balance screen: the account list/tree, its rows, and the [`BalanceView`]
//! that owns that state and its transitions.
//!
//! [`BalanceView`] is the balance-screen counterpart to
//! [`super::register::RegisterView`]. It always lives on the [`super::app::App`]
//! (the register screen is a transient drill-in over it), holds the account
//! tree, the derived rows, the display mode and fold state, and the active
//! account search, and exposes the pure transitions the dispatcher in
//! [`super::app::App::update`] delegates to.

use std::cmp::min;
use std::collections::HashSet;

#[cfg(test)]
use okane_core::report::Account;
use okane_core::report::{AccountAggregate, AccountTreeKey, Amount, BalanceTreeNode};

use crate::ui::table::TableNav;

use super::register::{OwnedRegisterScope, RegisterScope};
use super::search::{
    Search, SearchDirection, SearchIntent, SearchMatch, SearchMode, SearchPhase,
};

/// Whether the balance screen shows a flat account list or the account tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Every posted account, alphabetically, showing its own amount.
    Flat,
    /// The account hierarchy, indented and foldable, showing subtree totals.
    Tree,
}

/// One visible row of the balance table.
///
/// Derived from a [`BalanceView`]'s tree of [`BalanceTreeNode`]s for the current
/// [`DisplayMode`] and fold state (see [`BalanceView::rebuild_rows`]). Stores the
/// typed amount from the report layer so rendering can reformat lazily under
/// different display contexts (currency conversion, commodity toggling, etc.)
/// without rebuilding the row vector.
#[derive(Debug, Clone)]
pub struct BalanceRow<'ctx> {
    /// Index of the backing node in [`BalanceView::tree`].
    pub node: usize,
    /// Associated account.
    /// Used for search, snapshot and reload restore.
    pub account: AccountTreeKey<'ctx>,
    /// Display label: the full path in [`DisplayMode::Flat`], the leaf segment
    /// in [`DisplayMode::Tree`].
    pub label: &'ctx str,
    /// Depth in the account tree (`1` for a top-level account). Drives the
    /// tree-view indentation.
    pub depth: u16,
    /// Amount to display: the node's own amount in flat view, its subtree
    /// total in tree view.
    pub amount: Amount<'ctx>,
    /// Whether the backing node has children (shows a fold marker in tree view).
    pub has_children: bool,
    /// Whether the backing node is currently folded.
    pub folded: bool,
    /// The scope of the register this balance row is associated with.
    /// It's impossible to derive from account as tree-view balance account would include decendants_of,
    /// while the flat-view balance will use exact.
    pub scope: RegisterScope<'ctx>,
}

impl<'ctx> BalanceRow<'ctx> {
    /// A flat-view row for a concrete account. Used by the state-machine
    /// tests, which don't carry a tree.
    #[cfg(test)]
    pub fn flat(account: Account<'ctx>, amount: Amount<'ctx>) -> Self {
        Self {
            node: 0,
            account: account.into(),
            label: account.as_str(),
            depth: 0,
            amount,
            has_children: false,
            folded: false,
            scope: RegisterScope::Single(account),
        }
    }

    /// Number of rendered lines this row occupies (>= 1).
    ///
    /// One line per commodity, with a `0` placeholder line for empty balances.
    pub fn line_count(&self) -> u16 {
        amount_line_count(&self.amount)
    }

    /// Returns the full name of the account.
    pub fn full_name(&self) -> &'ctx str {
        account_full_label(&self.account)
    }
}

/// Printable full label of the AccountTeeKey.
pub fn account_full_label<'ctx>(account: &AccountTreeKey<'ctx>) -> &'ctx str {
    match account {
        AccountTreeKey::Root => "(total)",
        AccountTreeKey::Descendant(account) => account.as_str(),
    }
}

/// Number of lines an [`Amount`] would render as in a table.
pub(super) fn amount_line_count(amount: &Amount<'_>) -> u16 {
    let n = amount.iter().count();
    n.clamp(1, u16::MAX as usize) as u16
}

/// Balance-screen state that survives a reload, captured by
/// [`BalanceView::snapshot`] and re-applied by [`BalanceView::restore`].
/// Everything is plain owned data — arena references would dangle across the
/// reload's arena reset, so accounts are kept by name and re-resolved against
/// the rebuilt session.
#[derive(Debug, Clone)]
pub struct BalanceSnapshot {
    /// Carried over so page-up/down keeps working before the first frame.
    viewport_height: u16,
    /// Selected account on the balance screen, by name.
    selected_account: Option<String>,
    /// Active search intent; the matches are recomputed on restore.
    search: Option<SearchIntent>,
    last_search: String,
    /// Flat vs tree display mode.
    mode: DisplayMode,
    /// Full names of the folded tree nodes, re-resolved to indices on restore.
    folded: Vec<String>,
}

/// State for the balance screen: the account tree, the rows derived from it,
/// the display mode and fold state, and the active account search.
#[derive(Debug)]
pub struct BalanceView<'ctx> {
    /// The whole account tree, alphabetical/pre-order (index 0 is the root).
    /// Empty in the pure state-machine tests, which drive [`Self::rows`]
    /// directly.
    pub tree: Vec<BalanceTreeNode<'ctx>>,
    /// Flat vs tree display mode.
    pub mode: DisplayMode,
    /// Indices into [`Self::tree`] of currently folded nodes (tree view only).
    pub folded: HashSet<usize>,
    /// The rows currently visible in the balance table, derived from
    /// [`Self::tree`] + [`Self::mode`] + [`Self::folded`] by
    /// [`Self::rebuild_rows`].
    pub rows: Vec<BalanceRow<'ctx>>,
    pub nav: TableNav,
    /// Active account search on the balance screen, if any.
    pub search: Option<Search>,
    /// Most recently used search pattern, recalled by an empty interactive
    /// search via `C-s`/`C-r`. Shared across modal and interactive searches.
    /// Not `Option` because empty string can represent empty state.
    pub last_search: String,
}

impl<'ctx> BalanceView<'ctx> {
    /// Builds a view from a [`BalanceTree`](okane_core::report::BalanceTree)'s
    /// nodes, starting in flat mode with the derived rows. An empty `tree`
    /// yields a view with no rows (used for the empty/error session).
    pub fn new(tree: Vec<BalanceTreeNode<'ctx>>) -> Self {
        let mut view = Self {
            tree,
            mode: DisplayMode::Flat,
            folded: HashSet::new(),
            rows: Vec::new(),
            nav: TableNav::new(0),
            search: None,
            last_search: String::new(),
        };
        view.rebuild_rows();
        view
    }

    /// Builds a view from pre-derived balance rows, with no backing tree.
    /// Test-only helper for driving the pure state machine; production always
    /// goes through [`Self::new`] with a real tree.
    #[cfg(test)]
    pub fn with_rows(rows: Vec<BalanceRow<'ctx>>) -> Self {
        let nav = TableNav::new(rows.len());
        Self {
            tree: Vec::new(),
            mode: DisplayMode::Flat,
            folded: HashSet::new(),
            rows,
            nav,
            search: None,
            last_search: String::new(),
        }
    }

    /// The currently-selected balance row, if any.
    fn selected_row(&self) -> Option<&BalanceRow<'ctx>> {
        let idx = self.nav.table_state.selected()?;
        self.rows.get(idx)
    }

    /// Full name of the currently-selected account, if any.
    fn selected_full_name(&self) -> Option<&'ctx str> {
        self.selected_row().map(|r| r.full_name())
    }

    /// The drill target of the currently-selected balance row, if any.
    pub(super) fn selected_scope(&self) -> Option<RegisterScope<'ctx>> {
        self.selected_row().map(|r| r.scope)
    }

    /// Recomputes [`Self::rows`] from the tree for the current mode and fold
    /// state, keeping the selection on the same account when possible (or its
    /// nearest visible ancestor when it was folded away).
    ///
    /// A no-op without a backing tree: the pure state-machine tests drive
    /// [`Self::rows`] directly through [`Self::with_rows`], and a real empty
    /// ledger has no rows to rebuild either.
    fn rebuild_rows(&mut self) {
        if self.tree.is_empty() {
            return;
        }
        // `&'ctx str`, not tied to `&self`, so it survives the rebuild below
        // without owning a copy.
        let prev = self.selected_full_name();
        let viewport_height = self.nav.viewport_height;
        self.rows = match self.mode {
            DisplayMode::Flat => self.build_flat_rows(),
            DisplayMode::Tree => self.build_tree_rows(),
        };
        self.nav = TableNav::new(self.rows.len());
        self.nav.viewport_height = viewport_height;
        if let Some(name) = prev {
            self.select_by_name(name);
        }
    }

    /// Flat rows: every posted account (non-zero own amount), full name, own
    /// amount. Ancestor-only nodes have a zero own amount and are skipped.
    fn build_flat_rows(&self) -> Vec<BalanceRow<'ctx>> {
        self.tree
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                let account = match node.account.as_aggregate()? {
                    AccountAggregate::Account(account) => account,
                    AccountAggregate::Ancestor(_) => return None,
                };
                Some(BalanceRow {
                    node: i,
                    account: account.into(),
                    label: account.as_str(),
                    depth: node.depth,
                    amount: node.self_amount.clone(),
                    // it's flat and impossible to have children.
                    has_children: false,
                    folded: false,
                    scope: RegisterScope::Single(account),
                })
            })
            .collect()
    }

    /// Tree rows: a pre-order walk that skips the root and jumps over a folded
    /// node's whole (contiguous) subtree. Each row shows the leaf label,
    /// indented by depth, and the subtree total.
    fn build_tree_rows(&self) -> Vec<BalanceRow<'ctx>> {
        let mut rows = Vec::new();
        let mut i = 1; // index 0 is the synthetic root, never shown.
        while i < self.tree.len() {
            let node = &self.tree[i];
            let Some(aggregate) = node.account.as_aggregate() else {
                i += 1;
                continue;
            };
            let folded = self.folded.contains(&i);
            let label = match node.account {
                AccountTreeKey::Root => "(total)",
                AccountTreeKey::Descendant(account) => account.last_segment(),
            };
            rows.push(BalanceRow {
                node: i,
                account: aggregate.into(),
                label,
                depth: node.depth,
                amount: node.subtree_amount.clone(),
                has_children: node.has_children(),
                folded,
                scope: RegisterScope::Subtree(aggregate),
            });
            if folded && node.has_children() {
                i = node.subtree_range().end;
            } else {
                i += 1;
            }
        }
        rows
    }

    /// Selects the row for `name`, or its nearest visible ancestor (the longest
    /// full-name prefix present), leaving the selection unchanged when neither
    /// exists.
    fn select_by_name(&mut self, name: &str) {
        if let Some(idx) = self.rows.iter().position(|r| r.full_name() == name) {
            self.nav.select(idx);
            return;
        }
        let mut best: Option<usize> = None;
        let mut best_len = 0;
        for (idx, row) in self.rows.iter().enumerate() {
            let prefix = row.full_name();
            let is_ancestor = name.len() > prefix.len()
                && name.starts_with(prefix)
                && name.as_bytes()[prefix.len()] == b':';
            if is_ancestor && prefix.len() > best_len {
                best = Some(idx);
                best_len = prefix.len();
            }
        }
        if let Some(idx) = best {
            self.nav.select(idx);
        }
    }

    /// Toggles between the flat list and the account tree, dropping any active
    /// search (row identity changes between modes, so stale match indices must
    /// not carry over the reshape) and rebuilding the rows.
    pub(super) fn toggle_tree(&mut self) {
        self.search = None;
        self.mode = match self.mode {
            DisplayMode::Flat => DisplayMode::Tree,
            DisplayMode::Tree => DisplayMode::Flat,
        };
        self.rebuild_rows();
    }

    /// Folds/unfolds the selected node (tree view only, and only when it has
    /// children), rebuilds the rows, then recomputes the active search.
    pub(super) fn fold_selected(&mut self) {
        self.toggle_fold_selected();
        self.recompute_search();
    }

    /// Folds or unfolds every node (tree view only), rebuilds the rows, then
    /// recomputes the active search.
    pub(super) fn fold_all(&mut self) {
        self.toggle_fold_all();
        self.recompute_search();
    }

    fn toggle_fold_selected(&mut self) {
        if self.mode != DisplayMode::Tree {
            return;
        }
        let Some(row) = self.selected_row() else {
            return;
        };
        if !row.has_children {
            return;
        }
        let node = row.node;
        if !self.folded.remove(&node) {
            self.folded.insert(node);
        }
        self.rebuild_rows();
    }

    fn toggle_fold_all(&mut self) {
        if self.mode != DisplayMode::Tree {
            return;
        }
        let foldable: Vec<usize> = self
            .tree
            .iter()
            .enumerate()
            .filter(|(i, node)| *i != 0 && node.has_children())
            .map(|(i, _)| i)
            .collect();
        let all_folded = foldable.iter().all(|i| self.folded.contains(i));
        if all_folded {
            self.folded.clear();
        } else {
            self.folded = foldable.into_iter().collect();
        }
        self.rebuild_rows();
    }

    /// Full names of the currently folded tree nodes (for snapshotting).
    fn folded_names(&self) -> Vec<String> {
        self.folded
            .iter()
            .filter_map(|&i| self.tree.get(i))
            .filter_map(|node| node.account.as_aggregate())
            .map(|aggregate| aggregate.as_str().to_owned())
            .collect()
    }

    /// Re-resolves folded node names (from a snapshot) to indices in the
    /// freshly-built tree.
    fn apply_folded_names(&mut self, names: &[String]) {
        let wanted: HashSet<&str> = names.iter().map(String::as_str).collect();
        self.folded = self
            .tree
            .iter()
            .enumerate()
            .filter(|(i, node)| {
                *i != 0
                    && node
                        .account
                        .as_aggregate()
                        .is_some_and(|aggregate| wanted.contains(aggregate.as_str()))
            })
            .map(|(i, _)| i)
            .collect();
    }

    /// Rebuilds a [`RegisterScope`] from a persisted [`OwnedRegisterScope`] by
    /// locating the node in the freshly-built tree. `None` when the account no
    /// longer exists (or a `Single` scope now resolves to an ancestor-only node).
    ///
    /// Falls back to a matching visible row when there is no backing tree (the
    /// state-machine tests), where a row still carries a usable scope.
    pub(super) fn resolve_scope(&self, scope: &OwnedRegisterScope) -> Option<RegisterScope<'ctx>> {
        let name = scope.name();
        if let Some(aggregate) = self
            .tree
            .iter()
            .filter_map(|node| node.account.as_aggregate())
            .find(|aggregate| aggregate.as_str() == name)
        {
            return match scope {
                OwnedRegisterScope::Single(_) => match aggregate {
                    AccountAggregate::Account(account) => Some(RegisterScope::Single(account)),
                    AccountAggregate::Ancestor(_) => None,
                },
                OwnedRegisterScope::Subtree(_) => Some(RegisterScope::Subtree(aggregate)),
            };
        }
        self.rows
            .iter()
            .find(|row| row.full_name() == name)
            .map(|row| row.scope)
    }

    /// Captures the balance-screen state that should survive a reload as owned
    /// data (no `'ctx` borrows).
    pub(super) fn snapshot(&self) -> BalanceSnapshot {
        BalanceSnapshot {
            viewport_height: self.nav.viewport_height,
            selected_account: self.selected_full_name().map(str::to_owned),
            search: self.search.as_ref().map(|s| s.intent.clone()),
            last_search: self.last_search.clone(),
            mode: self.mode,
            folded: self.folded_names(),
        }
    }

    /// Restores a [`BalanceSnapshot`] into this freshly-built view: the display
    /// mode and fold state are reapplied first (they reshape the rows), then
    /// the selection follows the previously selected account (or the closest
    /// one by name when it disappeared), and any active search is recomputed
    /// against the new rows.
    pub(super) fn restore(&mut self, snapshot: &BalanceSnapshot) {
        self.mode = snapshot.mode;
        self.apply_folded_names(&snapshot.folded);
        self.rebuild_rows();

        self.nav.viewport_height = snapshot.viewport_height;
        if let Some(prev) = &snapshot.selected_account
            && let Some(idx) = restore_index(prev, &self.rows)
        {
            self.nav.select(idx);
        }

        self.last_search = snapshot.last_search.clone();
        if let Some(intent) = &snapshot.search {
            let mut intent = intent.clone();
            intent.origin = min(intent.origin, self.rows.len().saturating_sub(1));
            let matches = SearchMatch::compute(&intent.input, &self.rows);
            self.search = Some(Search { intent, matches });
        }
    }

    /// Opens a search of the given style, recording the current selection as
    /// the origin.
    pub(super) fn start_search(&mut self, mode: SearchMode, dir: SearchDirection) {
        let origin = self.nav.table_state.selected().unwrap_or(0);
        self.search = Some(Search {
            intent: SearchIntent {
                mode,
                dir,
                input: String::new(),
                no_previous: false,
                origin,
            },
            matches: None,
        });
    }

    /// Appends a character to the active search pattern and recomputes matches.
    pub(super) fn search_push(&mut self, c: char) {
        if let Some(search) = self.search.as_mut() {
            search.intent.input.push(c);
            search.intent.no_previous = false;
        }
        self.recompute_search();
    }

    /// Removes the last character from the active search pattern and recomputes.
    pub(super) fn search_pop(&mut self) {
        if let Some(search) = self.search.as_mut() {
            search.intent.input.pop();
            search.intent.no_previous = false;
        }
        self.recompute_search();
    }

    /// Fixes the current pattern (modal incremental → fixed); an empty pattern
    /// exits the search instead.
    pub(super) fn search_submit(&mut self) {
        match &self.search {
            // If empty pattern submitted, simply exits the search mode.
            Some(s) if s.intent.input.is_empty() => self.search = None,
            Some(search) => {
                self.last_search = search.intent.input.clone();
                if let Some(search) = self.search.as_mut()
                    && let SearchMode::Modal(phase) = &mut search.intent.mode
                {
                    *phase = SearchPhase::Fixed;
                }
            }
            None => {}
        }
    }

    /// Cancels an editing search: restores the origin selection and exits.
    pub(super) fn search_cancel(&mut self) {
        if let Some(search) = self.search.take() {
            // on cancel, search query won't be saved.
            self.nav.select(search.intent.origin);
        }
    }

    /// Closes the search, keeping the current selection.
    pub(super) fn search_close(&mut self) {
        self.search = None;
    }

    /// Next match (modal `n`); or, for interactive search, repeat forward /
    /// recall the previous pattern when empty (`C-s`).
    pub(super) fn search_next(&mut self) {
        self.search_or_recall(SearchDirection::Forward);
    }

    /// Previous match (modal `N`); or, for interactive search, repeat backward
    /// / recall the previous pattern when empty (`C-r`).
    pub(super) fn search_prev(&mut self) {
        self.search_or_recall(SearchDirection::Backward);
    }

    /// Ends an active interactive search, keeping the current selection. Used
    /// by keys that both navigate and leave i-search (`C-n`/`C-p`, Enter). A
    /// no-op for modal searches, which stay active during navigation.
    pub(super) fn end_interactive_search(&mut self) {
        if self
            .search
            .as_ref()
            .is_some_and(|s| matches!(s.intent.mode, SearchMode::Interactive))
            // clear search with take().
            && let Some(search) = self.search.take()
            && !search.intent.input.is_empty()
        {
            self.last_search = search.intent.input;
        }
    }

    /// Handles `C-s`/`C-r` (and modal `n`/`N`). An interactive search on an
    /// empty pattern recalls the last-used pattern (canonical isearch);
    /// otherwise it steps to the next/previous match.
    fn search_or_recall(&mut self, dir: SearchDirection) {
        let Some(search) = &mut self.search else {
            return;
        };
        // update direction before operation
        search.intent.dir = dir;
        let recall =
            search.intent.mode == SearchMode::Interactive && search.intent.input.is_empty();
        if recall {
            self.recall_last_search();
        } else {
            self.search_step();
        }
    }

    /// Restores [`Self::last_search`] into the active interactive search and
    /// jumps in `dir`. With no previous pattern, flips on the
    /// `[no previous search text]` notice and waits for input.
    fn recall_last_search(&mut self) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        search.intent.input = self.last_search.clone();
        search.intent.no_previous = self.last_search.is_empty();
        self.recompute_search();
    }

    /// Moves the balance selection to the next/previous match (wrapping). For
    /// an interactive search this also records `dir` so subsequent input keeps
    /// jumping the same way. No-op without matches.
    fn search_step(&mut self) {
        let Some(search) = self.search.as_ref() else {
            return;
        };
        let Some(Ok(m)) = search.matches.as_ref() else {
            return;
        };
        let current = self.nav.table_state.selected().unwrap_or(0);
        let Some(next) = m.step(current, search.intent.dir) else {
            return;
        };
        self.nav.select(next);
    }

    /// Recompiles the search pattern, recollects matching balance-row indices,
    /// and jumps the selection to the first match in the active direction.
    ///
    /// Modal searches always jump relative to the fixed origin; interactive
    /// searches jump relative to the current point, mirroring isearch. No-op
    /// when no search is active.
    pub(super) fn recompute_search(&mut self) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        let intent = &search.intent;
        let origin = intent.origin;
        let reference = match intent.mode {
            SearchMode::Modal(_) => origin,
            SearchMode::Interactive => self.nav.table_state.selected().unwrap_or(origin),
        };
        let matches = SearchMatch::compute(&intent.input, &self.rows);
        let jump = match &matches {
            Some(Ok(m)) => m.first_match(reference, intent.dir),
            _ => None,
        };
        search.matches = matches;
        if let Some(idx) = jump {
            self.nav.select(idx);
        }
    }
}

/// Row index to restore after a reload: the row of `prev_name` when it still
/// exists, otherwise the alphabetically closest row (insertion point, clamped
/// to the end). `None` when `rows` is empty.
///
/// Relies on `rows` being sorted by account name, which is the order
/// `Balance::into_vec` produces.
pub(super) fn restore_index(prev_name: &str, rows: &[BalanceRow<'_>]) -> Option<usize> {
    let last = rows.len().checked_sub(1)?;
    let idx = rows
        .binary_search_by(|r| r.full_name().cmp(prev_name))
        .unwrap_or_else(|insertion| insertion);
    Some(min(idx, last))
}
