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
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
#[cfg(test)]
use okane_core::report::Account;
use okane_core::report::{AccountAggregate, AccountTreeKey, Amount, BalanceTreeNode};

use crate::ui::keys::is_ctrl;
use crate::ui::table::{NavCommand, TableNav, key_to_nav};

use super::register::{OwnedRegisterScope, RegisterScope};
use super::render::amount_line_count;
use super::search::{
    Search, SearchDirection, SearchIntent, SearchMatch, SearchMode, SearchPhase, Translator,
};

/// Whether the balance screen shows a flat account list or the account tree.
///
/// Both modes open with the [total row](BalanceView::total_row): in
/// [`Self::Tree`] it is the tree's root, and a flat list is that same root
/// followed by its accounts rather than its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Every posted account, alphabetically, showing its own amount.
    Flat,
    /// The account hierarchy, indented and foldable, showing subtree totals.
    Tree,
}

/// Label of the total row — the whole ledger's balance, which is the tree's
/// synthetic root rather than an account, so it is named rather than addressed.
pub const TOTAL_LABEL: &str = "(total)";

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
    /// in [`DisplayMode::Tree`], and [`TOTAL_LABEL`] for the total row.
    pub label: &'ctx str,
    /// Depth in the account tree (`1` for a top-level account, `0` for the
    /// total row). Drives the tree-view indentation.
    pub depth: u16,
    /// Amount to display: the node's own amount in flat view, its subtree
    /// total in tree view — and, for the total row, the whole ledger's.
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
        AccountTreeKey::Root => TOTAL_LABEL,
        AccountTreeKey::Descendant(account) => account.as_str(),
    }
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

/// Messages handled by the balance screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceMessage {
    /// Move the selection within the balance table.
    Nav(NavCommand),
    /// Toggle between the flat list and the account tree (`t`).
    ToggleTree,
    /// Fold/unfold the selected tree node (`space`).
    ToggleFold,
    /// Fold or unfold every node in the tree (`x`).
    ToggleFoldAll,
    /// Drill into the selected balance row's register.
    OpenRegister,
    /// Open the modal (`/`) search bar (incremental phase).
    StartModalSearch,
    /// Open an interactive (`C-s`/`C-r`) search in the given direction.
    StartISearch(SearchDirection),
    /// Append a character to the search pattern.
    SearchPush(char),
    /// Remove the last character from the search pattern.
    SearchPop,
    /// Fix the current pattern (modal incremental → fixed); empty pattern exits.
    SearchSubmit,
    /// Cancel an editing search: restore the origin selection and exit.
    SearchCancel,
    /// Close the search: keep the current selection.
    SearchClose,
    /// Next match (modal `n` / interactive `C-s`).
    SearchNext,
    /// Previous match (modal `N` / interactive `C-r`).
    SearchPrev,
}

/// Effect a [`BalanceView`] asks the [`App`](super::app::App) to perform;
/// everything else is handled inside the view.
#[derive(Debug, Clone, Copy)]
pub enum BalanceAction<'ctx> {
    /// Drill into the register for `scope` (App loads the rows via the Ledger).
    OpenRegister { scope: RegisterScope<'ctx> },
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
    /// Width of the Amount column, cached because deciding it means formatting
    /// every row's amount — the one thing a frame cannot do per viewport, and
    /// the event loop redraws on a timer as well as on every key. Cleared by
    /// [`Self::rebuild_rows`], the only thing that changes what is measured;
    /// the display context it is measured against outlives the whole view.
    pub amount_width: Option<u16>,
    /// Active account search on the balance screen, if any.
    pub search: Option<Search>,
    /// Most recently used search pattern, recalled by an empty interactive
    /// search via `C-s`/`C-r`. Shared across modal and interactive searches.
    /// Not `Option` because empty string can represent empty state.
    pub last_search: String,
    /// How a typed pattern becomes the regex the rows are matched against.
    /// Shared with the session rather than owned, so the migemo process is
    /// spawned once and survives the reloads that rebuild this view.
    pub translator: Rc<Translator>,
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
            amount_width: None,
            search: None,
            last_search: String::new(),
            translator: Rc::default(),
        };
        view.rebuild_rows();
        view
    }

    /// Builds a view from pre-derived balance rows, with no backing tree.
    /// Test-only helper for driving the pure state machine; production always
    /// goes through [`Self::new`] with a real tree.
    #[cfg(test)]
    pub fn with_rows(rows: Vec<BalanceRow<'ctx>>) -> Self {
        let nav = TableNav::with_lines(rows.iter().map(BalanceRow::line_count));
        Self {
            tree: Vec::new(),
            mode: DisplayMode::Flat,
            folded: HashSet::new(),
            rows,
            nav,
            amount_width: None,
            search: None,
            last_search: String::new(),
            translator: Rc::default(),
        }
    }

    /// The currently-selected balance row, if any.
    fn selected_row(&self) -> Option<&BalanceRow<'ctx>> {
        self.rows.get(self.nav.selected_item()?)
    }

    /// Full name of the currently-selected account, if any.
    fn selected_full_name(&self) -> Option<&'ctx str> {
        self.selected_row().map(|r| r.full_name())
    }

    /// The drill target of the currently-selected balance row, if any.
    fn selected_scope(&self) -> Option<RegisterScope<'ctx>> {
        self.selected_row().map(|r| r.scope)
    }

    /// Applies a [`BalanceMessage`], returning a [`BalanceAction`] for the
    /// cross-screen transitions the view cannot perform itself.
    pub(super) fn update(&mut self, msg: BalanceMessage) -> Option<BalanceAction<'ctx>> {
        match msg {
            BalanceMessage::Nav(cmd) => {
                self.end_interactive_search();
                self.nav.apply(cmd);
            }
            BalanceMessage::OpenRegister => {
                if let Some(scope) = self.selected_scope() {
                    // An interactive search terminates on the register view transition.
                    self.end_interactive_search();
                    return Some(BalanceAction::OpenRegister { scope });
                }
            }
            BalanceMessage::StartModalSearch => self.start_search(
                SearchMode::Modal(SearchPhase::Incremental),
                SearchDirection::Forward,
            ),
            BalanceMessage::StartISearch(dir) => self.start_search(SearchMode::Interactive, dir),
            BalanceMessage::SearchPush(c) => self.search_push(c),
            BalanceMessage::SearchPop => self.search_pop(),
            BalanceMessage::SearchSubmit => self.search_submit(),
            BalanceMessage::SearchCancel => self.search_cancel(),
            BalanceMessage::SearchClose => self.search_close(),
            BalanceMessage::SearchNext => self.search_next(),
            BalanceMessage::SearchPrev => self.search_prev(),
            BalanceMessage::ToggleTree => self.toggle_tree(),
            BalanceMessage::ToggleFold => self.fold_selected(),
            BalanceMessage::ToggleFoldAll => self.fold_all(),
        }
        None
    }

    /// Translates a key event into a [`BalanceMessage`]. The editing search
    /// phases own every key; otherwise navigation and the balance actions map
    /// as usual. Global keys (quit, reload) return `None` for [`super::event`]
    /// to handle.
    pub(super) fn key_to_message(&self, key: KeyEvent) -> Option<BalanceMessage> {
        let ctrl = is_ctrl(key.modifiers);

        // Search capture. The editing phases (modal incremental, interactive
        // i-search) own every key; the modal fixed phase intercepts only its
        // own controls and lets the rest fall through so full navigation (and
        // Enter-to-register) keep working.
        if let Some(search) = &self.search {
            match search.intent.mode {
                SearchMode::Modal(SearchPhase::Incremental) => {
                    return match key.code {
                        KeyCode::Esc => Some(BalanceMessage::SearchCancel),
                        KeyCode::Enter => Some(BalanceMessage::SearchSubmit),
                        KeyCode::Backspace => Some(BalanceMessage::SearchPop),
                        KeyCode::Char(c) if !ctrl => Some(BalanceMessage::SearchPush(c)),
                        _ => None,
                    };
                }
                SearchMode::Modal(SearchPhase::Fixed) => match key.code {
                    KeyCode::Esc => return Some(BalanceMessage::SearchClose),
                    KeyCode::Char('n') => return Some(BalanceMessage::SearchNext),
                    KeyCode::Char('N') => return Some(BalanceMessage::SearchPrev),
                    _ => {} // fall through to normal UI
                },
                SearchMode::Interactive => match key.code {
                    KeyCode::Char('s') if ctrl => return Some(BalanceMessage::SearchNext),
                    KeyCode::Char('r') if ctrl => return Some(BalanceMessage::SearchPrev),
                    KeyCode::Char('g') if ctrl => return Some(BalanceMessage::SearchCancel),
                    KeyCode::Esc => return Some(BalanceMessage::SearchCancel),
                    KeyCode::Backspace => return Some(BalanceMessage::SearchPop),
                    KeyCode::Char(c) if !ctrl => return Some(BalanceMessage::SearchPush(c)),
                    _ => {} // fall through to normal UI
                },
            }
        }

        // Shared table navigation.
        if let Some(cmd) = key_to_nav(key) {
            return Some(BalanceMessage::Nav(cmd));
        }

        // Balance-specific actions.
        match key.code {
            KeyCode::Char('/') => Some(BalanceMessage::StartModalSearch),
            KeyCode::Char('s') if ctrl => {
                Some(BalanceMessage::StartISearch(SearchDirection::Forward))
            }
            KeyCode::Char('r') if ctrl => {
                Some(BalanceMessage::StartISearch(SearchDirection::Backward))
            }
            KeyCode::Char('t') => Some(BalanceMessage::ToggleTree),
            KeyCode::Char(' ') => Some(BalanceMessage::ToggleFold),
            KeyCode::Char('x') => Some(BalanceMessage::ToggleFoldAll),
            KeyCode::Enter => Some(BalanceMessage::OpenRegister),
            // q/Esc (quit) and r/F5 (reload) are handled by `super::event`.
            _ => None,
        }
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
        self.nav = TableNav::with_lines(self.rows.iter().map(BalanceRow::line_count));
        self.nav.viewport_height = viewport_height;
        // Different rows, so a different widest amount.
        self.amount_width = None;
        if let Some(name) = prev {
            self.select_by_name(name);
        }
    }

    /// The whole ledger's balance as a row, from the tree's synthetic root:
    /// the first row of either mode, so the number every account row is a part
    /// of is read before them rather than hunted for at the end.
    ///
    /// Unlike a parent row it is never foldable — folding it would hide every
    /// account behind the one row that cannot be scrolled away from — and it
    /// drills into the register of every account ([`RegisterScope::All`]), the
    /// same relationship a tree row has with its subtree.
    ///
    /// `None` for a ledger with no accounts, where a lone zero would say less
    /// than the "no balances" notice it would replace.
    fn total_row(&self) -> Option<BalanceRow<'ctx>> {
        let root = self.tree.first()?;
        if !root.has_children() {
            return None;
        }
        Some(BalanceRow {
            node: 0,
            account: AccountTreeKey::Root,
            label: TOTAL_LABEL,
            depth: 0,
            amount: root.subtree_amount.clone(),
            has_children: false,
            folded: false,
            scope: RegisterScope::All,
        })
    }

    /// Flat rows: the total, then every posted account (non-zero own amount),
    /// full name, own amount. Ancestor-only nodes have a zero own amount and
    /// are skipped.
    fn build_flat_rows(&self) -> Vec<BalanceRow<'ctx>> {
        let accounts = self.tree.iter().enumerate().filter_map(|(i, node)| {
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
        });
        self.total_row().into_iter().chain(accounts).collect()
    }

    /// Tree rows: the root as the total row, then a pre-order walk of its
    /// descendants that jumps over a folded node's whole (contiguous) subtree.
    /// Each row shows the leaf label, indented by depth, and the subtree total.
    fn build_tree_rows(&self) -> Vec<BalanceRow<'ctx>> {
        let mut rows: Vec<BalanceRow<'ctx>> = self.total_row().into_iter().collect();
        let mut i = 1; // index 0 is the synthetic root, drawn as the total row.
        while i < self.tree.len() {
            let node = &self.tree[i];
            let Some(aggregate) = node.account.as_aggregate() else {
                i += 1;
                continue;
            };
            let folded = self.folded.contains(&i);
            let label = aggregate.last_segment();
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
            self.nav.select_item(idx);
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
            self.nav.select_item(idx);
        }
    }

    /// Toggles between the flat list and the account tree, dropping any active
    /// search (row identity changes between modes, so stale match indices must
    /// not carry over the reshape) and rebuilding the rows.
    fn toggle_tree(&mut self) {
        self.search = None;
        self.mode = match self.mode {
            DisplayMode::Flat => DisplayMode::Tree,
            DisplayMode::Tree => DisplayMode::Flat,
        };
        self.rebuild_rows();
    }

    /// Folds/unfolds the selected node (tree view only, and only when it has
    /// children), rebuilds the rows, then recomputes the active search.
    fn fold_selected(&mut self) {
        self.toggle_fold_selected();
        self.recompute_search();
    }

    /// Folds or unfolds every node (tree view only), rebuilds the rows, then
    /// recomputes the active search.
    fn fold_all(&mut self) {
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
        // The total names no account, so there is nothing to lose to a reload.
        if matches!(scope, OwnedRegisterScope::All) {
            return Some(RegisterScope::All);
        }
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
                // Handled above, before any account lookup.
                OwnedRegisterScope::All => Some(RegisterScope::All),
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
            self.nav.select_item(idx);
        }

        self.last_search = snapshot.last_search.clone();
        if let Some(intent) = &snapshot.search {
            let mut intent = intent.clone();
            intent.origin = min(intent.origin, self.rows.len().saturating_sub(1));
            let matches = SearchMatch::compute(&intent.input, &self.rows, &self.translator);
            self.search = Some(Search { intent, matches });
        }
    }

    /// Opens a search of the given style, recording the current selection as
    /// the origin.
    fn start_search(&mut self, mode: SearchMode, dir: SearchDirection) {
        let origin = self.nav.selected_item().unwrap_or(0);
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
    fn search_push(&mut self, c: char) {
        if let Some(search) = self.search.as_mut() {
            search.intent.input.push(c);
            search.intent.no_previous = false;
        }
        self.recompute_search();
    }

    /// Removes the last character from the active search pattern and recomputes.
    fn search_pop(&mut self) {
        if let Some(search) = self.search.as_mut() {
            search.intent.input.pop();
            search.intent.no_previous = false;
        }
        self.recompute_search();
    }

    /// Fixes the current pattern (modal incremental → fixed); an empty pattern
    /// exits the search instead.
    fn search_submit(&mut self) {
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
    fn search_cancel(&mut self) {
        if let Some(search) = self.search.take() {
            // on cancel, search query won't be saved.
            self.nav.select_item(search.intent.origin);
        }
    }

    /// Closes the search, keeping the current selection.
    fn search_close(&mut self) {
        self.search = None;
    }

    /// Next match (modal `n`); or, for interactive search, repeat forward /
    /// recall the previous pattern when empty (`C-s`).
    fn search_next(&mut self) {
        self.search_or_recall(SearchDirection::Forward);
    }

    /// Previous match (modal `N`); or, for interactive search, repeat backward
    /// / recall the previous pattern when empty (`C-r`).
    fn search_prev(&mut self) {
        self.search_or_recall(SearchDirection::Backward);
    }

    /// Ends an active interactive search, keeping the current selection. Used
    /// by keys that both navigate and leave i-search (`C-n`/`C-p`, Enter). A
    /// no-op for modal searches, which stay active during navigation.
    fn end_interactive_search(&mut self) {
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
        let current = self.nav.selected_item().unwrap_or(0);
        let Some(next) = m.step(current, search.intent.dir) else {
            return;
        };
        self.nav.select_item(next);
    }

    /// Recompiles the search pattern, recollects matching balance-row indices,
    /// and jumps the selection to the first match in the active direction.
    ///
    /// Modal searches always jump relative to the fixed origin; interactive
    /// searches jump relative to the current point, mirroring isearch. No-op
    /// when no search is active.
    fn recompute_search(&mut self) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        let intent = &search.intent;
        let origin = intent.origin;
        let reference = match intent.mode {
            SearchMode::Modal(_) => origin,
            SearchMode::Interactive => self.nav.selected_item().unwrap_or(origin),
        };
        let matches = SearchMatch::compute(&intent.input, &self.rows, &self.translator);
        let jump = match &matches {
            Some(Ok(m)) => m.first_match(reference, intent.dir),
            _ => None,
        };
        search.matches = matches;
        if let Some(idx) = jump {
            self.nav.select_item(idx);
        }
    }
}

/// Row index to restore after a reload: the row of `prev_name` when it still
/// exists, otherwise the alphabetically closest row (insertion point, clamped
/// to the end). `None` when `rows` is empty.
///
/// Relies on `rows` being sorted by account name, which is the order
/// `Balance::into_vec` produces — and which the leading [`TOTAL_LABEL`] row
/// keeps, `(` sorting ahead of the letters an account name starts with.
pub(super) fn restore_index(prev_name: &str, rows: &[BalanceRow<'_>]) -> Option<usize> {
    let last = rows.len().checked_sub(1)?;
    let idx = rows
        .binary_search_by(|r| r.full_name().cmp(prev_name))
        .unwrap_or_else(|insertion| insertion);
    Some(min(idx, last))
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use bumpalo::Bump;
    use crossterm::event::KeyModifiers;
    use indoc::indoc;
    use okane_core::report::ReportContext;
    use rust_decimal_macros::dec;

    use super::super::testing::process;

    const ACCOUNTS: &[&str] = &[
        "Assets:Bank",      // 0
        "Assets:Cash",      // 1
        "Expenses:Food",    // 2
        "Income:Salary",    // 3
        "Liabilities:Card", // 4
    ];

    /// A ledger with a small nested hierarchy:
    /// `Assets:Bank:Checking`, `Assets:Cash`, `Expenses:Food`, `Equity`.
    const TREE_LEDGER: &str = indoc! {"
        2024/01/01 Init
            Assets:Bank:Checking    10 USD
            Assets:Cash    5 USD
            Expenses:Food    3 USD
            Equity
    "};

    /// Balance rows for `names`, in order, with zero amounts (no backing tree).
    fn rows_of<'ctx>(ctx: &ReportContext<'ctx>, names: &[&str]) -> Vec<BalanceRow<'ctx>> {
        names
            .iter()
            .map(|n| BalanceRow::flat(ctx.account(n).unwrap(), Amount::zero()))
            .collect()
    }

    /// A [`BalanceView`] whose rows are `names`, in order, with no backing tree.
    fn view_from_names<'ctx>(
        arena: &'ctx Bump,
        names: &[&str],
    ) -> (ReportContext<'ctx>, BalanceView<'ctx>) {
        let mut content = String::from("2024/01/01 Init\n");
        for name in names {
            content.push_str(&format!("    {name}    1 USD\n"));
        }
        content.push_str("    Equity\n");
        let (ctx, _ledger) = process(arena, &content);
        let view = BalanceView::with_rows(rows_of(&ctx, names));
        (ctx, view)
    }

    /// A tree-backed [`BalanceView`] from ledger `content`, in flat mode.
    fn tree_view<'ctx>(
        arena: &'ctx Bump,
        content: &str,
    ) -> (ReportContext<'ctx>, BalanceView<'ctx>) {
        use okane_core::report::BalanceTree;
        use okane_core::report::query::BalanceQuery;

        let (ctx, mut ledger) = process(arena, content);
        let balance = ledger
            .balance(&ctx, &BalanceQuery::default())
            .unwrap()
            .into_owned();
        let tree = BalanceTree::create(&ctx, balance).unwrap().into_nodes();
        let view = BalanceView::new(tree);
        (ctx, view)
    }

    /// The selected *account* (an index into `view.rows`), which is what every
    /// test below means — the table itself has one row per commodity line.
    fn selected(view: &BalanceView<'_>) -> Option<usize> {
        view.nav.selected_item()
    }

    fn full_names(view: &BalanceView<'_>) -> Vec<String> {
        view.rows.iter().map(|r| r.full_name().to_owned()).collect()
    }

    fn row_labels(view: &BalanceView<'_>) -> Vec<String> {
        view.rows.iter().map(|r| r.label.to_owned()).collect()
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    // --- search behaviour (BalanceView::update) ---

    #[test]
    fn start_search_records_origin() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::Nav(NavCommand::Down));
        view.update(BalanceMessage::Nav(NavCommand::Down));
        assert_eq!(selected(&view), Some(2));
        view.update(BalanceMessage::StartModalSearch);
        let search = view.search.as_ref().expect("search active");
        assert_eq!(
            search.intent.mode,
            SearchMode::Modal(SearchPhase::Incremental)
        );
        assert_eq!(search.intent.origin, 2);
        assert!(search.intent.input.is_empty());
        assert!(search.matched_rows().is_empty());
    }

    #[test]
    fn incremental_jumps_to_first_match_at_or_after_origin() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::Nav(NavCommand::Down));
        view.update(BalanceMessage::StartModalSearch);
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        let search = view.search.as_ref().unwrap();
        assert_eq!(search.matched_rows(), [0, 1]);
        assert_matches!(search.err(), None);
        assert_eq!(selected(&view), Some(1));
    }

    #[test]
    fn incremental_wraps_when_no_match_after_origin() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        for _ in 0..3 {
            view.update(BalanceMessage::Nav(NavCommand::Down));
        }
        view.update(BalanceMessage::StartModalSearch);
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        assert_eq!(view.search.as_ref().unwrap().matched_rows(), [0, 1]);
        assert_eq!(selected(&view), Some(0));
    }

    #[test]
    fn incremental_invalid_regex_sets_error() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        view.update(BalanceMessage::SearchPush('['));
        let search = view.search.as_ref().unwrap();
        assert_matches!(search.err(), Some(_));
        assert!(search.matched_rows().is_empty());
    }

    #[test]
    fn backspace_recomputes_matches() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        for c in "cash".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        assert_eq!(view.search.as_ref().unwrap().matched_rows(), [1]);
        view.update(BalanceMessage::SearchPop);
        view.update(BalanceMessage::SearchPop);
        assert_eq!(view.search.as_ref().unwrap().matched_rows(), [1, 4]);
    }

    #[test]
    fn submit_empty_exits_search() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        view.update(BalanceMessage::SearchSubmit);
        assert!(view.search.is_none());
    }

    #[test]
    fn submit_nonempty_enters_fixed_phase() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        view.update(BalanceMessage::SearchPush('a'));
        view.update(BalanceMessage::SearchSubmit);
        assert_eq!(
            view.search.as_ref().unwrap().intent.mode,
            SearchMode::Modal(SearchPhase::Fixed)
        );
    }

    #[test]
    fn isearch_forward_jumps_and_repeats() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        let search = view.search.as_ref().unwrap();
        assert_eq!(search.intent.mode, SearchMode::Interactive);
        assert_eq!(search.intent.dir, SearchDirection::Forward);
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        assert_eq!(view.search.as_ref().unwrap().matched_rows(), [0, 1]);
        assert_eq!(selected(&view), Some(0));
        view.update(BalanceMessage::SearchNext);
        assert_eq!(selected(&view), Some(1));
        view.update(BalanceMessage::SearchNext);
        assert_eq!(selected(&view), Some(0));
    }

    #[test]
    fn isearch_backward_jumps_to_last_match() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::Nav(NavCommand::Last));
        view.update(BalanceMessage::StartISearch(SearchDirection::Backward));
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        assert_eq!(selected(&view), Some(1));
        view.update(BalanceMessage::SearchPrev);
        assert_eq!(selected(&view), Some(0));
    }

    #[test]
    fn isearch_repeat_direction_steers_later_input() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(
            &arena,
            &["Assets:A", "Bonds:x", "Assets:B", "Bonds:y", "Assets:C"],
        );
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        assert_eq!(selected(&view), Some(0));
        view.update(BalanceMessage::SearchPrev);
        assert_eq!(selected(&view), Some(4));
        view.update(BalanceMessage::SearchPop);
        assert_eq!(selected(&view), Some(4));
    }

    #[test]
    fn isearch_cancel_restores_origin() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::Nav(NavCommand::Down));
        view.update(BalanceMessage::Nav(NavCommand::Down));
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        view.update(BalanceMessage::SearchCancel);
        assert!(view.search.is_none());
        assert_eq!(selected(&view), Some(2));
    }

    #[test]
    fn search_pattern_is_remembered_for_recall() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        for c in "salary".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        view.update(BalanceMessage::SearchSubmit);
        view.update(BalanceMessage::SearchClose);
        assert_eq!(&view.last_search, "salary");

        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        view.update(BalanceMessage::SearchNext);
        let search = view.search.as_ref().unwrap();
        assert_eq!(search.intent.input, "salary");
        assert!(!search.intent.no_previous);
        assert_eq!(search.matched_rows(), [3]);
        assert_eq!(selected(&view), Some(3));
    }

    #[test]
    fn isearch_recall_without_history_shows_notice() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        view.update(BalanceMessage::SearchNext);
        let search = view.search.as_ref().unwrap();
        assert!(search.intent.no_previous);
        assert!(search.intent.input.is_empty());
        view.update(BalanceMessage::SearchPush('a'));
        assert!(!view.search.as_ref().unwrap().intent.no_previous);
    }

    #[test]
    fn isearch_move_ends_search_and_navigates() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        view.update(BalanceMessage::Nav(NavCommand::Down));
        assert!(view.search.is_none());
        assert_eq!(selected(&view), Some(1));
        assert_eq!(&view.last_search, "assets");
    }

    #[test]
    fn isearch_open_register_ends_search() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        for c in "salary".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        let action = view.update(BalanceMessage::OpenRegister);
        assert_matches!(action, Some(BalanceAction::OpenRegister { .. }));
        assert!(view.search.is_none());
        assert_eq!(selected(&view), Some(3));
    }

    #[test]
    fn modal_fixed_search_survives_navigation() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        view.update(BalanceMessage::SearchSubmit);
        view.update(BalanceMessage::Nav(NavCommand::Down));
        assert!(view.search.is_some());
    }

    #[test]
    fn isearch_recall_backward_sets_direction() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.last_search = "assets".to_owned();
        view.update(BalanceMessage::Nav(NavCommand::Last));
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        view.update(BalanceMessage::SearchPrev);
        let search = view.search.as_ref().unwrap();
        assert_eq!(search.intent.input, "assets");
        assert_eq!(search.intent.mode, SearchMode::Interactive);
        assert_eq!(search.intent.dir, SearchDirection::Backward);
        assert_eq!(selected(&view), Some(1));
    }

    #[test]
    fn cancel_restores_origin() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::Nav(NavCommand::Down));
        view.update(BalanceMessage::Nav(NavCommand::Down));
        view.update(BalanceMessage::StartModalSearch);
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        assert_eq!(selected(&view), Some(0));
        view.update(BalanceMessage::SearchCancel);
        assert!(view.search.is_none());
        assert_eq!(selected(&view), Some(2));
    }

    #[test]
    fn close_keeps_selection() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        for c in "salary".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        view.update(BalanceMessage::SearchSubmit);
        assert_eq!(selected(&view), Some(3));
        view.update(BalanceMessage::SearchClose);
        assert!(view.search.is_none());
        assert_eq!(selected(&view), Some(3));
    }

    #[test]
    fn search_next_prev_wrap_over_matches() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        for c in "assets".chars() {
            view.update(BalanceMessage::SearchPush(c));
        }
        view.update(BalanceMessage::SearchSubmit);
        assert_eq!(selected(&view), Some(0));
        view.update(BalanceMessage::SearchNext);
        assert_eq!(selected(&view), Some(1));
        view.update(BalanceMessage::SearchNext);
        assert_eq!(selected(&view), Some(0));
        view.update(BalanceMessage::SearchPrev);
        assert_eq!(selected(&view), Some(1));
    }

    // --- amount / restore-index free functions ---

    #[test]
    fn amount_line_count_zero_amount_is_one() {
        let amount = Amount::zero();
        assert_eq!(amount_line_count(&amount), 1);
    }

    #[test]
    fn amount_line_count_matches_commodity_count() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodity_store_mut().ensure("USD");
        let eur = ctx.commodity_store_mut().ensure("EUR");
        let one = Amount::from_value(usd, dec!(1));
        let two = Amount::from_value(usd, dec!(1)) + Amount::from_value(eur, dec!(2));
        assert_eq!(amount_line_count(&one), 1);
        assert_eq!(amount_line_count(&two), 2);
    }

    #[test]
    fn restore_index_prefers_exact_match() {
        let arena = Bump::new();
        let (ctx, _view) = view_from_names(&arena, ACCOUNTS);
        let rows = rows_of(&ctx, ACCOUNTS);
        assert_eq!(restore_index("Expenses:Food", &rows), Some(2));
    }

    #[test]
    fn restore_index_falls_back_to_insertion_point() {
        let arena = Bump::new();
        let (ctx, _view) = view_from_names(&arena, ACCOUNTS);
        let rows = rows_of(&ctx, ACCOUNTS);
        assert_eq!(restore_index("Assets:Extra", &rows), Some(2));
        assert_eq!(restore_index("Aaa", &rows), Some(0));
        assert_eq!(restore_index("Zzz", &rows), Some(4));
    }

    #[test]
    fn restore_index_empty_rows_is_none() {
        let rows: &[BalanceRow<'_>] = &[];
        assert_eq!(restore_index("Assets:Bank", rows), None);
    }

    // --- tree / fold (BalanceView::update) ---

    #[test]
    fn flat_mode_shows_the_total_then_posted_accounts() {
        let arena = Bump::new();
        let (_ctx, view) = tree_view(&arena, TREE_LEDGER);
        assert_eq!(
            full_names(&view),
            [
                "(total)",
                "Assets:Bank:Checking",
                "Assets:Cash",
                "Equity",
                "Expenses:Food"
            ]
        );
    }

    /// The total row is the tree's root in either mode: the sum of every
    /// account, first, and drilling into it opens the whole ledger's register.
    #[test]
    fn total_row_sums_every_account_in_both_modes() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        let mut sum: Amount<'_> = view.rows[1..]
            .iter()
            .fold(Amount::zero(), |acc, row| acc + row.amount.clone());
        // The tree drops commodities that net out; the plain sum keeps them.
        sum.remove_zero_entries();

        for mode in [DisplayMode::Flat, DisplayMode::Tree] {
            if view.mode != mode {
                view.update(BalanceMessage::ToggleTree);
            }
            let total = &view.rows[0];
            assert_eq!(total.full_name(), "(total)");
            assert_eq!(total.label, "(total)");
            assert_eq!(total.depth, 0);
            assert_eq!(total.amount, sum, "in {mode:?}");
            // Never foldable: it would hide every row behind the one row that
            // is always on screen.
            assert!(!total.has_children);
        }
    }

    /// `space` on the total row does nothing — the whole tree would vanish.
    #[test]
    fn total_row_does_not_fold() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.update(BalanceMessage::ToggleTree);
        view.nav.select_item(0); // (total)
        view.update(BalanceMessage::ToggleFold);
        assert!(view.folded.is_empty());
        assert_eq!(view.rows.len(), 8);
    }

    #[test]
    fn total_open_register_drills_into_every_account() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.nav.select_item(0); // (total)
        let action = view.update(BalanceMessage::OpenRegister);
        assert_matches!(
            action,
            Some(BalanceAction::OpenRegister {
                scope: RegisterScope::All
            })
        );
    }

    /// A ledger with no accounts has no total to show either: a lone zero row
    /// would displace the "no balances" notice with less information.
    #[test]
    fn no_accounts_means_no_total_row() {
        let arena = Bump::new();
        let (_ctx, view) = tree_view(&arena, "");
        assert!(view.rows.is_empty());
    }

    #[test]
    fn toggle_tree_shows_hierarchy_with_leaf_labels() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        let checking = view.rows[1].amount.clone();
        let cash = view.rows[2].amount.clone();
        view.update(BalanceMessage::ToggleTree);
        assert_eq!(view.mode, DisplayMode::Tree);
        assert_eq!(
            full_names(&view),
            [
                "(total)",
                "Assets",
                "Assets:Bank",
                "Assets:Bank:Checking",
                "Assets:Cash",
                "Equity",
                "Expenses",
                "Expenses:Food",
            ]
        );
        assert_eq!(
            row_labels(&view),
            [
                "(total)", "Assets", "Bank", "Checking", "Cash", "Equity", "Expenses", "Food"
            ]
        );
        assert_eq!(view.rows[1].depth, 1);
        assert!(view.rows[1].has_children);
        assert_eq!(view.rows[3].depth, 3);
        assert!(!view.rows[3].has_children);
        assert_eq!(view.rows[1].amount, checking + cash);
        view.update(BalanceMessage::ToggleTree);
        assert_eq!(view.mode, DisplayMode::Flat);
        assert_eq!(view.rows.len(), 5);
    }

    #[test]
    fn toggle_fold_hides_selected_subtree() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.update(BalanceMessage::ToggleTree);
        view.nav.select_item(1); // Assets
        view.update(BalanceMessage::ToggleFold);
        assert_eq!(
            full_names(&view),
            ["(total)", "Assets", "Equity", "Expenses", "Expenses:Food"]
        );
        assert!(view.rows[1].folded);
        assert_eq!(selected(&view), Some(1));
        view.update(BalanceMessage::ToggleFold);
        assert!(!view.rows[1].folded);
        assert_eq!(view.rows.len(), 8);
    }

    #[test]
    fn toggle_fold_all_collapses_then_expands() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.update(BalanceMessage::ToggleTree);
        view.update(BalanceMessage::ToggleFoldAll);
        // The total survives fold-all: it is not one of the foldable nodes.
        assert_eq!(
            full_names(&view),
            ["(total)", "Assets", "Equity", "Expenses"]
        );
        view.update(BalanceMessage::ToggleFoldAll);
        assert_eq!(view.rows.len(), 8);
    }

    #[test]
    fn tree_open_register_drills_into_subtree() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.update(BalanceMessage::ToggleTree);
        view.nav.select_item(1); // Assets
        let action = view.update(BalanceMessage::OpenRegister);
        assert_matches!(
            action,
            Some(BalanceAction::OpenRegister { scope: RegisterScope::Subtree(agg) }) if agg.as_str() == "Assets"
        );
    }

    #[test]
    fn flat_open_register_drills_into_single_account() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.nav.select_item(2); // Assets:Cash
        let action = view.update(BalanceMessage::OpenRegister);
        assert_matches!(
            action,
            Some(BalanceAction::OpenRegister { scope: RegisterScope::Single(account) }) if account.as_str() == "Assets:Cash"
        );
    }

    #[test]
    fn open_register_with_no_selection_is_none() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, &[]);
        assert_matches!(view.update(BalanceMessage::OpenRegister), None);
    }

    #[test]
    fn tree_state_survives_snapshot_restore() {
        let arena = Bump::new();
        let (_ctx, mut view) = tree_view(&arena, TREE_LEDGER);
        view.update(BalanceMessage::ToggleTree);
        view.nav.select_item(1); // Assets
        view.update(BalanceMessage::ToggleFold); // fold Assets
        let snapshot = view.snapshot();

        let (_ctx2, mut view2) = tree_view(&arena, TREE_LEDGER);
        view2.restore(&snapshot);
        assert_eq!(view2.mode, DisplayMode::Tree);
        assert_eq!(
            full_names(&view2),
            ["(total)", "Assets", "Equity", "Expenses", "Expenses:Food"]
        );
        assert!(view2.rows[1].folded);
    }

    /// The total row's scope names no account, so a reload can never lose it.
    #[test]
    fn total_scope_survives_snapshot_restore() {
        let arena = Bump::new();
        let (_ctx, view) = tree_view(&arena, TREE_LEDGER);
        let owned = view.rows[0].scope.as_owned_scope();
        assert_matches!(
            view.resolve_scope(&owned),
            Some(RegisterScope::All),
            "the total row should resolve without an account to look up"
        );
    }

    // --- key mapping (BalanceView::key_to_message) ---

    #[test]
    fn key_maps_nav_keys() {
        let arena = Bump::new();
        let (_ctx, view) = view_from_names(&arena, ACCOUNTS);
        assert_eq!(
            view.key_to_message(key(KeyCode::Down)),
            Some(BalanceMessage::Nav(NavCommand::Down))
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('k'))),
            Some(BalanceMessage::Nav(NavCommand::Up))
        );
        assert_eq!(
            view.key_to_message(ctrl_key('n')),
            Some(BalanceMessage::Nav(NavCommand::Down))
        );
        assert_eq!(
            view.key_to_message(ctrl_key('p')),
            Some(BalanceMessage::Nav(NavCommand::Up))
        );
    }

    #[test]
    fn key_maps_balance_actions() {
        let arena = Bump::new();
        let (_ctx, view) = view_from_names(&arena, ACCOUNTS);
        assert_eq!(
            view.key_to_message(key(KeyCode::Enter)),
            Some(BalanceMessage::OpenRegister)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('/'))),
            Some(BalanceMessage::StartModalSearch)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('t'))),
            Some(BalanceMessage::ToggleTree)
        );
        assert_eq!(
            view.key_to_message(ctrl_key('s')),
            Some(BalanceMessage::StartISearch(SearchDirection::Forward))
        );
        assert_eq!(
            view.key_to_message(ctrl_key('r')),
            Some(BalanceMessage::StartISearch(SearchDirection::Backward))
        );
    }

    #[test]
    fn key_leaves_quit_and_reload_unmapped() {
        let arena = Bump::new();
        let (_ctx, view) = view_from_names(&arena, ACCOUNTS);
        // q/Esc and r/F5 are global; the event layer handles them.
        assert_eq!(view.key_to_message(key(KeyCode::Char('q'))), None);
        assert_eq!(view.key_to_message(key(KeyCode::Esc)), None);
        assert_eq!(view.key_to_message(key(KeyCode::Char('r'))), None);
        assert_eq!(view.key_to_message(key(KeyCode::F(5))), None);
    }

    #[test]
    fn key_incremental_search_captures_editing_keys() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('j'))),
            Some(BalanceMessage::SearchPush('j'))
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Backspace)),
            Some(BalanceMessage::SearchPop)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Enter)),
            Some(BalanceMessage::SearchSubmit)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Esc)),
            Some(BalanceMessage::SearchCancel)
        );
        // 'r' during editing is input, not reload.
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('r'))),
            Some(BalanceMessage::SearchPush('r'))
        );
    }

    #[test]
    fn key_fixed_search_intercepts_only_its_controls() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartModalSearch);
        view.update(BalanceMessage::SearchPush('a'));
        view.update(BalanceMessage::SearchSubmit); // fixed
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('n'))),
            Some(BalanceMessage::SearchNext)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('N'))),
            Some(BalanceMessage::SearchPrev)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Esc)),
            Some(BalanceMessage::SearchClose)
        );
        // Everything else falls through to normal navigation / drill-in.
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('j'))),
            Some(BalanceMessage::Nav(NavCommand::Down))
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Enter)),
            Some(BalanceMessage::OpenRegister)
        );
        // 'r' in fixed search is unmapped (event layer reloads).
        assert_eq!(view.key_to_message(key(KeyCode::Char('r'))), None);
    }

    #[test]
    fn key_interactive_search_captures_keys() {
        let arena = Bump::new();
        let (_ctx, mut view) = view_from_names(&arena, ACCOUNTS);
        view.update(BalanceMessage::StartISearch(SearchDirection::Forward));
        assert_eq!(
            view.key_to_message(key(KeyCode::Char('j'))),
            Some(BalanceMessage::SearchPush('j'))
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Backspace)),
            Some(BalanceMessage::SearchPop)
        );
        assert_eq!(
            view.key_to_message(ctrl_key('s')),
            Some(BalanceMessage::SearchNext)
        );
        assert_eq!(
            view.key_to_message(ctrl_key('r')),
            Some(BalanceMessage::SearchPrev)
        );
        assert_eq!(
            view.key_to_message(ctrl_key('g')),
            Some(BalanceMessage::SearchCancel)
        );
        assert_eq!(
            view.key_to_message(key(KeyCode::Esc)),
            Some(BalanceMessage::SearchCancel)
        );
        // RET drills in; C-n/C-p move — all end the search at update time.
        assert_eq!(
            view.key_to_message(key(KeyCode::Enter)),
            Some(BalanceMessage::OpenRegister)
        );
        assert_eq!(
            view.key_to_message(ctrl_key('n')),
            Some(BalanceMessage::Nav(NavCommand::Down))
        );
        assert_eq!(
            view.key_to_message(ctrl_key('p')),
            Some(BalanceMessage::Nav(NavCommand::Up))
        );
    }
}
