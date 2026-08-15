//! Register drill-down screen: its rows and the scope it is filtered to.

use std::cmp::max;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent};
use okane_core::report::query::{Conversion, DateRange};
use okane_core::report::{Account, AccountAggregate, Amount};

use crate::ui::table::{NavCommand, TableNav, key_to_nav};

use super::render::amount_line_count;

/// What a register drill-in from the balance screen should match.
#[derive(Debug, Clone, Copy)]
pub enum RegisterScope<'ctx> {
    /// Exactly one account (flat-view drill-in).
    Single(Account<'ctx>),
    /// An account and all of its descendants (tree-view drill-in).
    Subtree(AccountAggregate<'ctx>),
}

impl<'ctx> RegisterScope<'ctx> {
    /// The account/prefix name shown in the register title.
    pub fn display_name(&self) -> &'ctx str {
        match self {
            RegisterScope::Single(account) => account.as_str(),
            RegisterScope::Subtree(aggregate) => aggregate.as_str(),
        }
    }

    /// Owned form that survives a reload (see [`OwnedRegisterScope`]).
    pub(super) fn as_owned_scope(self) -> OwnedRegisterScope {
        match self {
            RegisterScope::Single(account) => {
                OwnedRegisterScope::Single(account.as_str().to_owned())
            }
            RegisterScope::Subtree(aggregate) => {
                OwnedRegisterScope::Subtree(aggregate.as_str().to_owned())
            }
        }
    }
}

/// Owned mirror of [`RegisterScope`] that survives a reload: the arena reset
/// invalidates the `'ctx` references, so the scope is kept by account name and
/// rebuilt with [`super::balance::BalanceView::resolve_scope`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedRegisterScope {
    /// A single account, by full name.
    Single(String),
    /// An account and all its descendants, by full name.
    Subtree(String),
}

impl OwnedRegisterScope {
    /// The account/prefix name the register was filtered to.
    pub(super) fn name(&self) -> &str {
        match self {
            OwnedRegisterScope::Single(name) | OwnedRegisterScope::Subtree(name) => name,
        }
    }
}

/// One row of the register table.
///
/// The account is implied by the active [`RegisterView`] (exact-match
/// filter), so it is not duplicated per row.
#[derive(Debug, Clone)]
pub struct RegisterRow<'ctx> {
    pub date: NaiveDate,
    pub payee: String,
    pub amount: Amount<'ctx>,
    pub total: Amount<'ctx>,
}

impl RegisterRow<'_> {
    /// Number of table rows this entry occupies (>= 1) — the taller of its two
    /// amount columns, since each contributes one line per commodity.
    pub fn line_count(&self) -> u16 {
        max(
            amount_line_count(&self.amount),
            amount_line_count(&self.total),
        )
    }
}

/// Query parameters reused for every register lookup during the session
/// (built once from the CLI's `EvalOptions`).
#[derive(Debug, Clone, Copy)]
pub struct RegisterQueryTemplate<'ctx> {
    pub conversion: Option<Conversion<'ctx>>,
    pub date_range: DateRange,
}

/// State for the register drill-down screen.
#[derive(Debug)]
pub struct RegisterView<'ctx> {
    /// What this register is filtered to (single account or subtree). Also the
    /// source of the title-bar label (see [`Self::title`]).
    pub scope: RegisterScope<'ctx>,
    pub rows: Vec<RegisterRow<'ctx>>,
    pub nav: TableNav,
    /// Cached `(amount, total)` column widths. The amounts are fixed for the
    /// life of the view, so the renderer computes these once (scanning all
    /// rows) on first draw and reuses them, keeping per-frame work
    /// proportional to the viewport rather than the row count.
    pub col_widths: Option<(u16, u16)>,
}

impl<'ctx> RegisterView<'ctx> {
    pub fn new(scope: RegisterScope<'ctx>, rows: Vec<RegisterRow<'ctx>>) -> Self {
        let mut nav = TableNav::with_lines(rows.iter().map(RegisterRow::line_count));
        // Most recent entry is the most useful starting point — its *last* row,
        // so the running total it carries is on screen whole. Landing on its
        // first row would pin that row to the bottom edge and cut the rest of
        // the entry off below it.
        nav.select_last_row();
        Self {
            scope,
            rows,
            nav,
            col_widths: None,
        }
    }

    /// The account/prefix name shown in the title bar (always the scope's name).
    pub fn title(&self) -> &'ctx str {
        self.scope.display_name()
    }

    /// Applies a [`RegisterMessage`], returning a [`RegisterAction`] for the
    /// cross-screen transitions the view cannot perform itself.
    pub(super) fn update(&mut self, msg: RegisterMessage) -> Option<RegisterAction> {
        match msg {
            RegisterMessage::Nav(cmd) => self.nav.apply(cmd),
            RegisterMessage::Leave => return Some(RegisterAction::Leave),
        }
        None
    }

    /// Translates a key event into a [`RegisterMessage`]. Global keys (`Ctrl-C`,
    /// reload) are left for [`super::event`], so `None` lets them fall through.
    pub(super) fn key_to_message(key: KeyEvent) -> Option<RegisterMessage> {
        if let Some(cmd) = key_to_nav(key) {
            return Some(RegisterMessage::Nav(cmd));
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(RegisterMessage::Leave),
            _ => None,
        }
    }
}

/// Messages handled by the register drill-down screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterMessage {
    /// Move the selection within the register table.
    Nav(NavCommand),
    /// Leave the register and return to the balance screen.
    Leave,
}

/// Effect a [`RegisterView`] asks the [`App`](super::app::App) to perform;
/// everything else is handled inside the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterAction {
    /// Return to the balance screen.
    Leave,
}

/// Snapshot of register view that survives a reload, as part of
/// [`super::app::UiSnapshot`].
#[derive(Debug, Clone)]
pub struct RegisterSnapshot {
    /// What the register was filtered to (single account or subtree), owned so
    /// it survives the arena reset.
    pub(super) scope: OwnedRegisterScope,
    /// Index of the selected entry — an index into
    /// [`RegisterView::rows`], not a table row, so it survives an entry
    /// whose line count changes with the data.
    pub(super) cursor: usize,
}

impl RegisterSnapshot {
    /// Captures a register view's scope (owned) and cursor.
    pub(super) fn capture(view: &RegisterView<'_>) -> Self {
        Self {
            scope: view.scope.as_owned_scope(),
            cursor: view.nav.selected_item().unwrap_or(0),
        }
    }

    pub(super) fn scope(&self) -> &OwnedRegisterScope {
        &self.scope
    }

    pub(super) fn cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use crossterm::event::KeyModifiers;

    use super::super::testing::make_account;

    /// A register view for `account` with `n` one-line rows, cursor on the last.
    fn register_view<'ctx>(account: Account<'ctx>, n: usize) -> RegisterView<'ctx> {
        let rows: Vec<RegisterRow<'ctx>> = (0..n)
            .map(|i| RegisterRow {
                date: NaiveDate::from_ymd_opt(2024, 1, (i + 1) as u32).unwrap(),
                payee: "payee".to_owned(),
                amount: Amount::zero(),
                total: Amount::zero(),
            })
            .collect();
        RegisterView::new(RegisterScope::Single(account), rows)
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn new_selects_last_entry() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let view = register_view(account, 3);
        assert_eq!(view.nav.selected_item(), Some(2));
    }

    #[test]
    fn nav_moves_and_clamps_selection() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut view = register_view(account, 3);
        view.update(RegisterMessage::Nav(NavCommand::First));
        assert_eq!(view.nav.selected_row(), 0);
        assert_eq!(view.update(RegisterMessage::Nav(NavCommand::Up)), None);
        assert_eq!(view.nav.selected_row(), 0);
        view.update(RegisterMessage::Nav(NavCommand::Down));
        assert_eq!(view.nav.selected_row(), 1);
        view.update(RegisterMessage::Nav(NavCommand::Last));
        assert_eq!(view.nav.selected_row(), 2);
    }

    #[test]
    fn leave_returns_action() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut view = register_view(account, 1);
        assert_eq!(
            view.update(RegisterMessage::Leave),
            Some(RegisterAction::Leave)
        );
    }

    #[test]
    fn key_to_message_maps_nav_and_leave() {
        assert_eq!(
            RegisterView::key_to_message(key(KeyCode::Down)),
            Some(RegisterMessage::Nav(NavCommand::Down))
        );
        assert_eq!(
            RegisterView::key_to_message(key(KeyCode::Char('k'))),
            Some(RegisterMessage::Nav(NavCommand::Up))
        );
        assert_eq!(
            RegisterView::key_to_message(ctrl_key('n')),
            Some(RegisterMessage::Nav(NavCommand::Down))
        );
        assert_eq!(
            RegisterView::key_to_message(key(KeyCode::Char('q'))),
            Some(RegisterMessage::Leave)
        );
        assert_eq!(
            RegisterView::key_to_message(key(KeyCode::Esc)),
            Some(RegisterMessage::Leave)
        );
    }

    #[test]
    fn key_to_message_leaves_global_keys_unmapped() {
        // Enter and reload keys fall through to the event layer.
        assert_eq!(RegisterView::key_to_message(key(KeyCode::Enter)), None);
        assert_eq!(RegisterView::key_to_message(key(KeyCode::Char('r'))), None);
        assert_eq!(RegisterView::key_to_message(key(KeyCode::F(5))), None);
    }
}
