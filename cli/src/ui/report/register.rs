//! Register drill-down screen: its rows, the scope it is filtered to, and the
//! [`Screen`] focus enum that selects between the balance and register views.

use std::cmp::max;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent};
use okane_core::report::query::{Conversion, DateRange};
use okane_core::report::{Account, AccountAggregate, Amount};

use crate::ui::keys::is_ctrl;
use crate::ui::table::TableNav;

use super::balance::amount_line_count;

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
    /// Number of rendered lines this row occupies (>= 1).
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
        let mut nav = TableNav::new(rows.len());
        // Most recent entry is the most useful starting point.
        nav.select_last();
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
            RegisterMessage::MoveUp => self.nav.move_selection(-1),
            RegisterMessage::MoveDown => self.nav.move_selection(1),
            RegisterMessage::PageUp => {
                let delta = -(self.nav.page_size() as isize);
                self.nav.move_selection(delta);
            }
            RegisterMessage::PageDown => {
                let delta = self.nav.page_size() as isize;
                self.nav.move_selection(delta);
            }
            RegisterMessage::SelectFirst => self.nav.select_first(),
            RegisterMessage::SelectLast => self.nav.select_last(),
            RegisterMessage::Leave => return Some(RegisterAction::Leave),
        }
        None
    }

    /// Translates a key event into a [`RegisterMessage`]. Global keys (`Ctrl-C`,
    /// reload) are left for [`super::event`], so `None` lets them fall through.
    pub(super) fn key_to_message(key: KeyEvent) -> Option<RegisterMessage> {
        let ctrl = is_ctrl(key.modifiers);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(RegisterMessage::MoveUp),
            KeyCode::Char('p') if ctrl => Some(RegisterMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(RegisterMessage::MoveDown),
            KeyCode::Char('n') if ctrl => Some(RegisterMessage::MoveDown),
            KeyCode::PageUp => Some(RegisterMessage::PageUp),
            KeyCode::Char('b') if ctrl => Some(RegisterMessage::PageUp),
            KeyCode::PageDown => Some(RegisterMessage::PageDown),
            KeyCode::Char('f') if ctrl => Some(RegisterMessage::PageDown),
            KeyCode::Home | KeyCode::Char('g') => Some(RegisterMessage::SelectFirst),
            KeyCode::End | KeyCode::Char('G') => Some(RegisterMessage::SelectLast),
            KeyCode::Char('q') | KeyCode::Esc => Some(RegisterMessage::Leave),
            _ => None,
        }
    }
}

/// Top-level screen the user is currently looking at.
#[derive(Debug)]
pub enum Screen<'ctx> {
    Balance,
    Register(RegisterView<'ctx>),
}

/// Messages handled by the register drill-down screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterMessage {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    SelectFirst,
    SelectLast,
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
    /// Selected cursor index.
    pub(super) cursor: usize,
}

impl RegisterSnapshot {
    /// Captures a register view's scope (owned) and cursor.
    pub(super) fn capture(view: &RegisterView<'_>) -> Self {
        Self {
            scope: view.scope.as_owned_scope(),
            cursor: view.nav.table_state.selected().unwrap_or(0),
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
    fn new_selects_last_row() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let view = register_view(account, 3);
        assert_eq!(view.nav.table_state.selected(), Some(2));
    }

    #[test]
    fn nav_moves_and_clamps_selection() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut view = register_view(account, 3);
        view.update(RegisterMessage::SelectFirst);
        assert_eq!(view.nav.table_state.selected(), Some(0));
        assert_eq!(view.update(RegisterMessage::MoveUp), None);
        assert_eq!(view.nav.table_state.selected(), Some(0));
        view.update(RegisterMessage::MoveDown);
        assert_eq!(view.nav.table_state.selected(), Some(1));
        view.update(RegisterMessage::SelectLast);
        assert_eq!(view.nav.table_state.selected(), Some(2));
    }

    #[test]
    fn leave_returns_action() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut view = register_view(account, 1);
        assert_eq!(view.update(RegisterMessage::Leave), Some(RegisterAction::Leave));
    }

    #[test]
    fn key_to_message_maps_nav_and_leave() {
        assert_eq!(
            RegisterView::key_to_message(key(KeyCode::Down)),
            Some(RegisterMessage::MoveDown)
        );
        assert_eq!(
            RegisterView::key_to_message(key(KeyCode::Char('k'))),
            Some(RegisterMessage::MoveUp)
        );
        assert_eq!(
            RegisterView::key_to_message(ctrl_key('n')),
            Some(RegisterMessage::MoveDown)
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
