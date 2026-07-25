//! UI application state.
//!
//! Follows The Elm Architecture: state lives in [`App`], all transitions go
//! through [`App::update`] driven by a [`Message`]. Key handling in
//! [`super::event`] translates raw `KeyEvent`s into messages based on the
//! currently active screen and overlay.
//!
//! [`App`] is a thin dispatcher over two screen states: the persistent
//! [`BalanceView`] (the account list/tree, always present) and the transient
//! [`RegisterView`] nested in [`Screen::Register`]. Balance-screen transitions
//! are delegated to [`BalanceView`]; [`App`] owns only the cross-screen
//! concerns — the focused screen, the modal overlay, the footer notice, and
//! the reload snapshot.

use std::cmp::min;

use crate::ui::table::TableNav;

use okane_core::report::BalanceTreeNode;

use super::balance::{BalanceSnapshot, BalanceView};
use super::overlay::{Overlay, ScrollDelta};
use super::register::{
    RegisterAction, RegisterMessage, RegisterQueryTemplate, RegisterRow, RegisterScope,
    RegisterSnapshot, RegisterView, Screen,
};
use super::search::{SearchDirection, SearchMode, SearchPhase};

/// Messages that drive state transitions (Elm-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    SelectFirst,
    SelectLast,
    // Balance-screen display toggles (flat/tree and folding).
    /// Toggle between the flat list and the account tree (`t`).
    ToggleTree,
    /// Fold/unfold the selected tree node (`space`).
    ToggleFold,
    /// Fold or unfold every node in the tree (`x`).
    ToggleFoldAll,
    /// User asked to drill into the selected balance row.
    OpenRegister,
    /// A message routed to the active register screen.
    Register(RegisterMessage),
    /// User asked to quit from balance — show the confirmation overlay.
    RequestQuit,
    /// Confirm quit from the overlay.
    ConfirmQuit,
    /// Dismiss the current overlay.
    DismissOverlay,
    /// Scroll the body of a scrollable overlay.
    OverlayScroll(ScrollDelta),
    /// Unconditional quit (Ctrl-C).
    QuitImmediate,
    /// Open the modal (`/`) balance search bar (incremental phase).
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
    /// Next match (modal `n`); or, for interactive search, repeat forward /
    /// recall the previous pattern when empty (`C-s`).
    SearchNext,
    /// Previous match (modal `N`); or, for interactive search, repeat backward
    /// / recall the previous pattern when empty (`C-r`).
    SearchPrev,
    /// Re-read the ledger data from disk (`r` / `F5`), keeping the UI state.
    Reload,
}

/// Effect requested by [`App::update`] that requires resources the pure
/// state machine does not own (here: `&mut Ledger` to compute a register).
#[derive(Debug, Clone, Copy)]
pub enum Command<'ctx> {
    LoadRegister {
        scope: RegisterScope<'ctx>,
    },
    /// Re-run the whole load/process pipeline and swap the data in.
    Reload,
}

/// UI state that survives a reload, captured by [`App::snapshot`] and
/// re-applied by [`App::restore`]. Everything is plain owned data — arena
/// references would dangle across the reload's arena reset, so accounts
/// are kept by name and re-resolved against the rebuilt session.
#[derive(Debug, Clone)]
pub struct UiSnapshot {
    /// Balance-screen state (selection, search, mode, fold state).
    balance: BalanceSnapshot,
    /// Snapshot of the register if it's the register screen.
    /// Extend this once [`Screen`] is more than 2 states.
    register: Option<RegisterSnapshot>,
}

/// Application state for the TUI session.
#[derive(Debug)]
pub struct App<'ctx> {
    pub source_display: String,
    /// Balance screen state — always present; the register screen is a
    /// transient drill-in drawn over it.
    pub balance: BalanceView<'ctx>,
    pub screen: Screen<'ctx>,
    pub overlay: Option<Overlay>,
    /// Transient one-line notice shown in the footer. Cleared on the next key
    /// press. Failures worth reading in full go to [`Overlay::Error`] instead,
    /// which is dismissed explicitly.
    pub error_toast: Option<String>,
    pub register_template: RegisterQueryTemplate<'ctx>,
    pub should_quit: bool,
}

impl<'ctx> App<'ctx> {
    /// Builds an app from a [`BalanceTree`](okane_core::report::BalanceTree)'s
    /// nodes, starting in flat mode with the derived rows. An empty `tree`
    /// yields an app with no rows (used for the empty/error session).
    pub fn new(
        source_display: String,
        tree: Vec<BalanceTreeNode<'ctx>>,
        register_template: RegisterQueryTemplate<'ctx>,
    ) -> Self {
        Self {
            source_display,
            balance: BalanceView::new(tree),
            screen: Screen::Balance,
            overlay: None,
            error_toast: None,
            register_template,
            should_quit: false,
        }
    }

    /// Builds an app from pre-derived balance rows, with no backing tree.
    /// Test-only helper for driving the pure state machine; production always
    /// goes through [`Self::new`] with a real tree.
    #[cfg(test)]
    pub fn with_rows(
        source_display: String,
        balance_rows: Vec<super::balance::BalanceRow<'ctx>>,
        register_template: RegisterQueryTemplate<'ctx>,
    ) -> Self {
        Self {
            source_display,
            balance: BalanceView::with_rows(balance_rows),
            screen: Screen::Balance,
            overlay: None,
            error_toast: None,
            register_template,
            should_quit: false,
        }
    }

    /// Mutable handle to whichever nav drives the currently visible table.
    fn active_nav_mut(&mut self) -> &mut TableNav {
        match &mut self.screen {
            Screen::Balance => &mut self.balance.nav,
            Screen::Register(view) => &mut view.nav,
        }
    }

    /// Applies a message; optionally returns a [`Command`] for the event
    /// loop to execute (the only impure step in this flow).
    pub fn update(&mut self, msg: Message) -> Option<Command<'ctx>> {
        // Any key press dismisses a transient error notice.
        self.error_toast = None;

        // QuitImmediate is honored regardless of overlay/screen.
        if matches!(msg, Message::QuitImmediate) {
            self.should_quit = true;
            return None;
        }

        if self.overlay.is_some() {
            match msg {
                Message::ConfirmQuit => self.should_quit = true,
                Message::DismissOverlay => self.overlay = None,
                Message::OverlayScroll(delta) => {
                    if let Some(Overlay::Error(popup)) = self.overlay.as_mut() {
                        popup.scroll(delta);
                    }
                }
                // Quitting from the error modal skips the dismiss step: fall
                // through so the quit prompt replaces it.
                Message::RequestQuit if matches!(self.overlay, Some(Overlay::Error(_))) => {
                    self.overlay = Some(Overlay::QuitConfirm);
                }
                // Retrying from the error modal: the reload rebuilds the whole
                // session (and this overlay with it).
                Message::Reload if matches!(self.overlay, Some(Overlay::Error(_))) => {
                    return Some(Command::Reload);
                }
                // Ignore other input while a modal is up.
                _ => {}
            }
            return None;
        }

        match msg {
            Message::MoveUp => {
                self.balance.end_interactive_search();
                self.active_nav_mut().move_selection(-1);
            }
            Message::MoveDown => {
                self.balance.end_interactive_search();
                self.active_nav_mut().move_selection(1);
            }
            Message::PageUp => {
                let nav = self.active_nav_mut();
                let delta = -(nav.page_size() as isize);
                nav.move_selection(delta);
            }
            Message::PageDown => {
                let nav = self.active_nav_mut();
                let delta = nav.page_size() as isize;
                nav.move_selection(delta);
            }
            Message::SelectFirst => self.active_nav_mut().select_first(),
            Message::SelectLast => self.active_nav_mut().select_last(),
            Message::OpenRegister => {
                if matches!(self.screen, Screen::Balance)
                    && let Some(scope) = self.balance.selected_scope()
                {
                    // An interactive search drills in like the normal view:
                    // end the search, keeping the cursor on the chosen account.
                    self.balance.end_interactive_search();
                    return Some(Command::LoadRegister { scope });
                }
            }
            Message::Register(register_msg) => {
                if let Screen::Register(view) = &mut self.screen {
                    match view.update(register_msg) {
                        Some(RegisterAction::Leave) => self.screen = Screen::Balance,
                        None => {}
                    }
                }
            }
            Message::RequestQuit => {
                if matches!(self.screen, Screen::Balance) {
                    self.overlay = Some(Overlay::QuitConfirm);
                }
            }
            // The screen guard mirrors the old inline guard: start a search on
            // the balance screen, or continue editing one that is already open.
            Message::StartModalSearch => {
                if matches!(self.screen, Screen::Balance) || self.balance.search.is_some() {
                    self.balance.start_search(
                        SearchMode::Modal(SearchPhase::Incremental),
                        SearchDirection::Forward,
                    );
                }
            }
            Message::StartISearch(dir) => {
                if matches!(self.screen, Screen::Balance) || self.balance.search.is_some() {
                    self.balance.start_search(SearchMode::Interactive, dir);
                }
            }
            Message::SearchPush(c) => self.balance.search_push(c),
            Message::SearchPop => self.balance.search_pop(),
            Message::SearchSubmit => self.balance.search_submit(),
            Message::SearchCancel => self.balance.search_cancel(),
            Message::SearchClose => self.balance.search_close(),
            Message::SearchNext => self.balance.search_next(),
            Message::SearchPrev => self.balance.search_prev(),
            Message::Reload => return Some(Command::Reload),
            Message::ToggleTree => {
                if matches!(self.screen, Screen::Balance) {
                    self.balance.toggle_tree();
                }
            }
            Message::ToggleFold => {
                if matches!(self.screen, Screen::Balance) {
                    self.balance.fold_selected();
                }
            }
            Message::ToggleFoldAll => {
                if matches!(self.screen, Screen::Balance) {
                    self.balance.fold_all();
                }
            }
            // Already handled above, or only meaningful while an overlay is up.
            Message::QuitImmediate
            | Message::ConfirmQuit
            | Message::DismissOverlay
            | Message::OverlayScroll(_) => {}
        }
        None
    }

    /// Called by the event loop once a [`Command::LoadRegister`] has been
    /// fulfilled.
    pub fn show_register(&mut self, scope: RegisterScope<'ctx>, rows: Vec<RegisterRow<'ctx>>) {
        self.screen = Screen::Register(RegisterView::new(scope, rows));
    }

    /// Like [`Self::show_register`], but restores a previous cursor position
    /// (clamped to the new row count) instead of jumping to the last entry.
    /// Used when re-entering the register after a reload.
    pub fn show_register_at(
        &mut self,
        scope: RegisterScope<'ctx>,
        rows: Vec<RegisterRow<'ctx>>,
        index: usize,
    ) {
        let mut view = RegisterView::new(scope, rows);
        if let Some(last) = view.nav.row_count.checked_sub(1) {
            view.nav.select(min(index, last));
        }
        self.screen = Screen::Register(view);
    }

    /// Captures the UI state that should survive a reload as owned data
    /// (no `'ctx` borrows): the whole session, arena included, is torn
    /// down before the snapshot is restored into the next one.
    pub fn snapshot(&self) -> UiSnapshot {
        UiSnapshot {
            balance: self.balance.snapshot(),
            register: match &self.screen {
                Screen::Balance => None,
                Screen::Register(view) => Some(RegisterSnapshot::capture(view)),
            },
        }
    }

    /// Restores a [`UiSnapshot`] into this freshly-built `App`: the balance
    /// screen is restored first (mode, fold state, selection, and search are
    /// reapplied against the new rows).
    ///
    /// When the snapshot had the register screen open, returns
    /// `Some((scope, index))` asking the caller to re-query that register and
    /// open it via [`Self::show_register_at`]. If the account no longer exists,
    /// stays on the balance screen (with a notice) and returns `None`.
    pub fn restore(&mut self, snapshot: &UiSnapshot) -> Option<(RegisterScope<'ctx>, usize)> {
        self.balance.restore(&snapshot.balance);

        let register = snapshot.register.as_ref()?;
        match self.balance.resolve_scope(register.scope()) {
            Some(register_scope) => Some((register_scope, register.cursor())),
            None => {
                let name = register.scope().name();
                self.error_toast = Some(format!(
                    "account {name} is gone after reload; back to balance"
                ));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use bumpalo::Bump;
    use chrono::NaiveDate;
    use indoc::indoc;
    use okane_core::report::{Account, Amount, ReportContext};
    use rust_decimal_macros::dec;

    use crate::ui::table::TableNav;

    use super::super::balance::{BalanceRow, DisplayMode, amount_line_count, restore_index};
    use super::super::overlay::ErrorPopup;
    use super::super::testing::{make_account, process, template};

    /// Build an `App` with no balance rows — sufficient for testing the
    /// pure state machine. (Constructing a `BalanceRow` requires an
    /// `Account<'ctx>`, whose interner has no public constructor outside
    /// `okane_core`, so we side-step it.)
    fn app_no_rows<'ctx>() -> App<'ctx> {
        App::new("test".to_owned(), Vec::new(), template())
    }

    /// Process a ledger containing `names` and return the context plus an
    /// `App` whose balance rows are those accounts, in order, with zero
    /// amounts. Row index `i` corresponds to `names[i]`.
    fn make_balance_app<'ctx>(
        arena: &'ctx Bump,
        names: &[&str],
    ) -> (ReportContext<'ctx>, App<'ctx>) {
        let mut content = String::from("2024/01/01 Init\n");
        for name in names {
            content.push_str(&format!("    {name}    1 USD\n"));
        }
        content.push_str("    Equity\n");
        let (ctx, _ledger) = process(arena, &content);
        let rows: Vec<BalanceRow> = names
            .iter()
            .map(|n| BalanceRow::flat(ctx.account(n).unwrap(), Amount::zero()))
            .collect();
        let app = App::with_rows("test".to_owned(), rows, template());
        (ctx, app)
    }

    const ACCOUNTS: &[&str] = &[
        "Assets:Bank",      // 0
        "Assets:Cash",      // 1
        "Expenses:Food",    // 2
        "Income:Salary",    // 3
        "Liabilities:Card", // 4
    ];

    fn selected(app: &App<'_>) -> Option<usize> {
        app.balance.nav.table_state.selected()
    }

    #[test]
    fn start_search_records_origin() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::MoveDown);
        app.update(Message::MoveDown);
        assert_eq!(selected(&app), Some(2));
        app.update(Message::StartModalSearch);
        let search = app.balance.search.as_ref().expect("search active");
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
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        // Origin at index 1.
        app.update(Message::MoveDown);
        app.update(Message::StartModalSearch);
        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }
        let search = app.balance.search.as_ref().unwrap();
        assert_eq!(search.matched_rows(), [0, 1]);
        assert_matches!(search.err(), None);
        // First match at-or-after origin 1 is 1.
        assert_eq!(selected(&app), Some(1));
    }

    #[test]
    fn incremental_wraps_when_no_match_after_origin() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        // Origin at index 3 — no "assets" match at-or-after, so wrap to 0.
        for _ in 0..3 {
            app.update(Message::MoveDown);
        }
        app.update(Message::StartModalSearch);
        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }
        assert_eq!(app.balance.search.as_ref().unwrap().matched_rows(), [0, 1]);
        assert_eq!(selected(&app), Some(0));
    }

    #[test]
    fn incremental_invalid_regex_sets_error() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        app.update(Message::SearchPush('['));
        let search = app.balance.search.as_ref().unwrap();
        assert_matches!(search.err(), Some(_));
        assert!(search.matched_rows().is_empty());
    }

    #[test]
    fn backspace_recomputes_matches() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        for c in "cash".chars() {
            app.update(Message::SearchPush(c));
        }
        assert_eq!(app.balance.search.as_ref().unwrap().matched_rows(), [1]);
        // Backspace down to "ca" — matches "Assets:Cash" and "Liabilities:Card".
        app.update(Message::SearchPop);
        app.update(Message::SearchPop);
        assert_eq!(app.balance.search.as_ref().unwrap().matched_rows(), [1, 4]);
    }

    #[test]
    fn submit_empty_exits_search() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        app.update(Message::SearchSubmit);
        assert!(app.balance.search.is_none());
    }

    #[test]
    fn submit_nonempty_enters_fixed_phase() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        app.update(Message::SearchPush('a'));
        app.update(Message::SearchSubmit);
        assert_eq!(
            app.balance.search.as_ref().unwrap().intent.mode,
            SearchMode::Modal(SearchPhase::Fixed)
        );
    }

    #[test]
    fn isearch_forward_jumps_and_repeats() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);

        app.update(Message::StartISearch(SearchDirection::Forward));

        let search = app.balance.search.as_ref().unwrap();
        assert_eq!(search.intent.mode, SearchMode::Interactive);
        assert_eq!(search.intent.dir, SearchDirection::Forward);

        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }

        // First forward match at-or-after origin 0.
        assert_eq!(app.balance.search.as_ref().unwrap().matched_rows(), [0, 1]);
        assert_eq!(selected(&app), Some(0));
        // C-s repeats forward, wrapping.
        app.update(Message::SearchNext);
        assert_eq!(selected(&app), Some(1));
        app.update(Message::SearchNext);
        assert_eq!(selected(&app), Some(0));
    }

    #[test]
    fn isearch_backward_jumps_to_last_match() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        // Start at the last row so a backward search lands on the prior match.
        app.update(Message::SelectLast);
        app.update(Message::StartISearch(SearchDirection::Backward));
        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }
        // Last match at-or-before origin 4 is index 1.
        assert_eq!(selected(&app), Some(1));
        // C-r repeats backward.
        app.update(Message::SearchPrev);
        assert_eq!(selected(&app), Some(0));
    }

    #[test]
    fn isearch_repeat_direction_steers_later_input() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(
            &arena,
            &["Assets:A", "Bonds:x", "Assets:B", "Bonds:y", "Assets:C"],
        );
        app.update(Message::StartISearch(SearchDirection::Forward));
        for c in "assets".chars() {
            app.update(Message::SearchPush(c)); // matches [0, 2, 4], at 0
        }
        assert_eq!(selected(&app), Some(0));
        app.update(Message::SearchPrev); // C-r → backward, wraps to last match 4
        assert_eq!(selected(&app), Some(4));
        // Backspace keeps the backward direction: from point 4, last match <= 4.
        app.update(Message::SearchPop); // "asset" still matches [0, 2, 4]
        assert_eq!(selected(&app), Some(4));
    }

    #[test]
    fn isearch_cancel_restores_origin() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::MoveDown);
        app.update(Message::MoveDown); // origin 2
        app.update(Message::StartISearch(SearchDirection::Forward));
        for c in "assets".chars() {
            app.update(Message::SearchPush(c)); // jumps to 0
        }
        app.update(Message::SearchCancel);
        assert!(app.balance.search.is_none());
        assert_eq!(selected(&app), Some(2));
    }

    #[test]
    fn search_pattern_is_remembered_for_recall() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        // Run and close a modal search to populate the last-used pattern.
        app.update(Message::StartModalSearch);
        for c in "salary".chars() {
            app.update(Message::SearchPush(c));
        }
        app.update(Message::SearchSubmit); // → fixed
        app.update(Message::SearchClose);
        assert_eq!(&app.balance.last_search, "salary");

        // A fresh interactive search with an empty pattern recalls it on C-s.
        app.update(Message::StartISearch(SearchDirection::Forward));
        app.update(Message::SearchNext);
        let search = app.balance.search.as_ref().unwrap();
        assert_eq!(search.intent.input, "salary");
        assert!(!search.intent.no_previous);
        assert_eq!(search.matched_rows(), [3]);
        assert_eq!(selected(&app), Some(3));
    }

    #[test]
    fn isearch_recall_without_history_shows_notice() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartISearch(SearchDirection::Forward));
        // No previous search: C-s flips on the notice and waits for input.
        app.update(Message::SearchNext);
        let search = app.balance.search.as_ref().unwrap();
        assert!(search.intent.no_previous);
        assert!(search.intent.input.is_empty());
        // Typing clears the notice and resumes a normal search.
        app.update(Message::SearchPush('a'));
        assert!(!app.balance.search.as_ref().unwrap().intent.no_previous);
    }

    #[test]
    fn isearch_move_ends_search_and_navigates() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartISearch(SearchDirection::Forward));
        for c in "assets".chars() {
            app.update(Message::SearchPush(c)); // matches [0, 1], selection 0
        }
        // C-n (MoveDown) ends the i-search and moves one row down.
        app.update(Message::MoveDown);
        assert!(app.balance.search.is_none());
        assert_eq!(selected(&app), Some(1));
        // The pattern is remembered for later recall.
        assert_eq!(&app.balance.last_search, "assets");
    }

    #[test]
    fn isearch_enter_opens_register_and_ends_search() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartISearch(SearchDirection::Forward));
        for c in "salary".chars() {
            app.update(Message::SearchPush(c)); // selection 3
        }
        let cmd = app.update(Message::OpenRegister);
        assert_matches!(cmd, Some(Command::LoadRegister { .. }));
        assert!(app.balance.search.is_none());
        assert_eq!(selected(&app), Some(3));
    }

    #[test]
    fn modal_fixed_search_survives_navigation() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }
        app.update(Message::SearchSubmit); // fixed
        // Unlike i-search, a modal search stays active during navigation.
        app.update(Message::MoveDown);
        assert!(app.balance.search.is_some());
    }

    #[test]
    fn isearch_recall_backward_sets_direction() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.balance.last_search = "assets".to_owned();
        app.update(Message::SelectLast); // origin 4
        app.update(Message::StartISearch(SearchDirection::Forward));
        // C-r on empty: recall + search backward from origin → last match (1).
        app.update(Message::SearchPrev);
        let search = app.balance.search.as_ref().unwrap();
        assert_eq!(search.intent.input, "assets");
        assert_eq!(search.intent.mode, SearchMode::Interactive);
        assert_eq!(search.intent.dir, SearchDirection::Backward);
        assert_eq!(selected(&app), Some(1));
    }

    #[test]
    fn cancel_restores_origin() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::MoveDown);
        app.update(Message::MoveDown); // origin = 2
        app.update(Message::StartModalSearch);
        for c in "assets".chars() {
            app.update(Message::SearchPush(c)); // jumps selection to 0
        }
        assert_eq!(selected(&app), Some(0));
        app.update(Message::SearchCancel);
        assert!(app.balance.search.is_none());
        assert_eq!(selected(&app), Some(2));
    }

    #[test]
    fn close_keeps_selection() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        for c in "salary".chars() {
            app.update(Message::SearchPush(c));
        }
        app.update(Message::SearchSubmit); // fixed; selection at the match (3)
        assert_eq!(selected(&app), Some(3));
        app.update(Message::SearchClose);
        assert!(app.balance.search.is_none());
        assert_eq!(selected(&app), Some(3));
    }

    #[test]
    fn search_next_prev_wrap_over_matches() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        for c in "assets".chars() {
            app.update(Message::SearchPush(c)); // matches [0, 1], selection 0
        }
        app.update(Message::SearchSubmit);
        assert_eq!(selected(&app), Some(0));
        app.update(Message::SearchNext);
        assert_eq!(selected(&app), Some(1));
        app.update(Message::SearchNext); // wrap
        assert_eq!(selected(&app), Some(0));
        app.update(Message::SearchPrev); // wrap backward
        assert_eq!(selected(&app), Some(1));
    }

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
    fn request_quit_on_balance_opens_overlay() {
        let mut app = app_no_rows();
        assert!(app.update(Message::RequestQuit).is_none());
        assert_eq!(app.overlay, Some(Overlay::QuitConfirm));
        assert!(!app.should_quit);
    }

    #[test]
    fn dismiss_overlay_keeps_session_alive() {
        let mut app = app_no_rows();
        app.update(Message::RequestQuit);
        app.update(Message::DismissOverlay);
        assert_eq!(app.overlay, None);
        assert!(!app.should_quit);
    }

    #[test]
    fn confirm_quit_from_overlay_quits() {
        let mut app = app_no_rows();
        app.update(Message::RequestQuit);
        app.update(Message::ConfirmQuit);
        assert!(app.should_quit);
    }

    #[test]
    fn quit_immediate_quits_from_any_state() {
        let mut app = app_no_rows();
        app.update(Message::RequestQuit);
        assert_eq!(app.overlay, Some(Overlay::QuitConfirm));
        app.update(Message::QuitImmediate);
        assert!(app.should_quit);
    }

    #[test]
    fn open_register_with_no_selection_is_noop() {
        let mut app = app_no_rows();
        assert!(app.update(Message::OpenRegister).is_none());
        assert!(matches!(app.screen, Screen::Balance));
    }

    #[test]
    fn nav_messages_ignored_while_overlay_visible() {
        let mut app = app_no_rows();
        // Pretend there are rows to move through by poking the nav directly.
        app.balance.nav = TableNav::new(3);
        app.update(Message::RequestQuit);
        app.update(Message::MoveDown);
        assert_eq!(app.balance.nav.table_state.selected(), Some(0));
    }

    fn popup(lines: usize, viewport_height: u16) -> ErrorPopup {
        let mut popup = ErrorPopup::new(
            "failed to load test.ledger".to_owned(),
            (0..lines).map(|i| format!("line {i}")).collect(),
        );
        popup.viewport_height = viewport_height;
        popup
    }

    fn app_with_error_modal<'ctx>() -> App<'ctx> {
        let mut app = app_no_rows();
        app.overlay = Some(Overlay::Error(popup(10, 4)));
        app
    }

    #[test]
    fn error_modal_scrolls_on_overlay_scroll() {
        let mut app = app_with_error_modal();
        assert!(
            app.update(Message::OverlayScroll(ScrollDelta::Bottom))
                .is_none()
        );
        assert_matches!(&app.overlay, Some(Overlay::Error(p)) if p.scroll == 6);
    }

    #[test]
    fn error_modal_survives_key_that_clears_footer_notice() {
        let mut app = app_with_error_modal();
        app.error_toast = Some("transient".to_owned());
        app.update(Message::MoveDown);
        assert!(app.error_toast.is_none());
        assert_matches!(app.overlay, Some(Overlay::Error(_)));
    }

    #[test]
    fn request_quit_replaces_error_modal_with_quit_prompt() {
        let mut app = app_with_error_modal();
        app.update(Message::RequestQuit);
        assert_eq!(app.overlay, Some(Overlay::QuitConfirm));
        assert!(!app.should_quit);
    }

    #[test]
    fn dismiss_closes_error_modal() {
        let mut app = app_with_error_modal();
        app.update(Message::DismissOverlay);
        assert_eq!(app.overlay, None);
    }

    #[test]
    fn reload_through_error_modal_returns_command() {
        let mut app = app_with_error_modal();
        assert_matches!(app.update(Message::Reload), Some(Command::Reload));
    }

    #[test]
    fn reload_during_quit_prompt_is_ignored() {
        let mut app = app_no_rows();
        app.update(Message::RequestQuit);
        assert!(app.update(Message::Reload).is_none());
        assert_eq!(app.overlay, Some(Overlay::QuitConfirm));
    }

    /// A single-account register screen for `account` with `nav`.
    fn register_screen<'ctx>(account: Account<'ctx>, nav: TableNav) -> Screen<'ctx> {
        Screen::Register(RegisterView {
            scope: RegisterScope::Single(account),
            rows: Vec::new(),
            nav,
            col_widths: None,
        })
    }

    #[test]
    fn leave_register_returns_to_balance() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let mut app = app_no_rows();
        // Bypass show_register's RegisterView::new — it just needs *some*
        // register screen state to flip the enum variant.
        app.screen = register_screen(account, TableNav::new(0));
        app.update(Message::Register(RegisterMessage::Leave));
        assert!(matches!(app.screen, Screen::Balance));
    }

    /// Balance rows for `names` (must be sorted, matching `Balance::into_vec`
    /// order) resolved against an existing context.
    fn rows_of<'ctx>(ctx: &ReportContext<'ctx>, names: &[&str]) -> Vec<BalanceRow<'ctx>> {
        names
            .iter()
            .map(|n| BalanceRow::flat(ctx.account(n).unwrap(), Amount::zero()))
            .collect()
    }

    #[test]
    fn restore_index_prefers_exact_match() {
        let arena = Bump::new();
        let (ctx, _app) = make_balance_app(&arena, ACCOUNTS);
        let rows = rows_of(&ctx, ACCOUNTS);
        assert_eq!(restore_index("Expenses:Food", &rows), Some(2));
    }

    #[test]
    fn restore_index_falls_back_to_insertion_point() {
        let arena = Bump::new();
        let (ctx, _app) = make_balance_app(&arena, ACCOUNTS);
        let rows = rows_of(&ctx, ACCOUNTS);
        // Between Assets:Cash (1) and Expenses:Food (2).
        assert_eq!(restore_index("Assets:Extra", &rows), Some(2));
        // Before every row.
        assert_eq!(restore_index("Aaa", &rows), Some(0));
        // Past the last row, clamped.
        assert_eq!(restore_index("Zzz", &rows), Some(4));
    }

    #[test]
    fn restore_index_empty_rows_is_none() {
        let rows: &[BalanceRow<'_>] = &[];
        assert_eq!(restore_index("Assets:Bank", rows), None);
    }

    #[test]
    fn reload_message_produces_command() {
        let mut app = app_no_rows();
        assert_matches!(app.update(Message::Reload), Some(Command::Reload));
    }

    #[test]
    fn any_key_clears_error_notice() {
        let mut app = app_no_rows();
        app.error_toast = Some("boom".to_owned());
        app.update(Message::MoveDown);
        assert_eq!(app.error_toast, None);
    }

    /// A fresh `App` over `names`, as if built for the next session.
    fn next_app<'ctx>(ctx: &ReportContext<'ctx>, names: &[&str]) -> App<'ctx> {
        App::with_rows("test".to_owned(), rows_of(ctx, names), template())
    }

    #[test]
    fn restore_follows_selected_account() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.balance.nav.select(2); // Expenses:Food
        let snapshot = app.snapshot();

        let mut app = next_app(
            &ctx,
            &[
                "Assets:Cash",
                "Expenses:Food",
                "Income:Salary",
                "Liabilities:Card",
            ],
        );
        assert_matches!(app.restore(&snapshot), None);
        // Expenses:Food moved from index 2 to 1.
        assert_eq!(selected(&app), Some(1));
    }

    #[test]
    fn restore_vanished_account_selects_closest() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.balance.nav.select(2); // Expenses:Food
        let snapshot = app.snapshot();

        let mut app = next_app(
            &ctx,
            &[
                "Assets:Bank",
                "Assets:Cash",
                "Income:Salary",
                "Liabilities:Card",
            ],
        );
        app.restore(&snapshot);
        // Expenses:Food is gone; the insertion point lands on Income:Salary.
        assert_eq!(selected(&app), Some(2));
    }

    #[test]
    fn restore_keeps_viewport_height() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.balance.nav.viewport_height = 12;
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, ACCOUNTS);
        app.restore(&snapshot);
        assert_eq!(app.balance.nav.viewport_height, 12);
    }

    #[test]
    fn restore_recomputes_search_matches() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }
        app.update(Message::SearchSubmit); // fixed
        assert_eq!(app.balance.search.as_ref().unwrap().matched_rows(), [0, 1]);
        app.balance.last_search = "salary".to_owned();
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, &["Assets:Bank", "Income:Salary"]);
        app.restore(&snapshot);
        let search = app.balance.search.as_ref().unwrap();
        assert_eq!(search.intent.input, "assets");
        assert_eq!(search.matched_rows(), [0]);
        // Origin is clamped into the new row range.
        assert!(search.intent.origin < 2);
        assert_eq!(&app.balance.last_search, "salary");
    }

    #[test]
    fn restore_requests_register_requery() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        let account = ctx.account("Assets:Cash").unwrap();
        let mut nav = TableNav::new(5);
        nav.select(3);
        app.screen = register_screen(account, nav);
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, ACCOUNTS);
        let got = app.restore(&snapshot);
        assert_matches!(got, Some((scope, 3)) if scope.display_name() == "Assets:Cash");
        // The screen stays on balance until the caller queries the rows and
        // opens the register via `show_register_at`.
        assert!(matches!(app.screen, Screen::Balance));
        assert_eq!(app.error_toast, None);
    }

    #[test]
    fn restore_register_account_vanished_falls_back() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        let account = ctx.account("Assets:Cash").unwrap();
        app.screen = register_screen(account, TableNav::new(0));
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, &["Assets:Bank", "Income:Salary"]);
        let got = app.restore(&snapshot);
        assert_matches!(got, None);
        assert!(matches!(app.screen, Screen::Balance));
        assert_matches!(&app.error_toast, Some(_));
    }

    #[test]
    fn show_register_at_clamps_index() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let rows: Vec<RegisterRow<'_>> = (0..3)
            .map(|i| RegisterRow {
                date: NaiveDate::from_ymd_opt(2024, 1, i + 1).unwrap(),
                payee: "payee".to_owned(),
                amount: Amount::zero(),
                total: Amount::zero(),
            })
            .collect();

        let mut app = app_no_rows();
        let scope = RegisterScope::Single(account);
        app.show_register_at(scope, rows.clone(), 10);
        let Screen::Register(view) = &app.screen else {
            panic!("expected register screen");
        };
        assert_eq!(view.nav.table_state.selected(), Some(2));

        app.show_register_at(scope, rows, 1);
        let Screen::Register(view) = &app.screen else {
            panic!("expected register screen");
        };
        assert_eq!(view.nav.table_state.selected(), Some(1));
    }

    #[test]
    fn request_quit_from_register_does_not_open_overlay() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let mut app = app_no_rows();
        app.screen = register_screen(account, TableNav::new(0));
        assert!(app.update(Message::RequestQuit).is_none());
        // From register, q/Esc leaves to balance (mapped at the event layer)
        // rather than opening the quit overlay.
        assert_eq!(app.overlay, None);
    }

    /// A ledger with a small nested hierarchy:
    /// `Assets:Bank:Checking`, `Assets:Cash`, `Expenses:Food`, `Equity`.
    const TREE_LEDGER: &str = indoc! {"
        2024/01/01 Init
            Assets:Bank:Checking    10 USD
            Assets:Cash    5 USD
            Expenses:Food    3 USD
            Equity
    "};

    /// Builds a tree-backed app from ledger `content`, in the flat default mode.
    fn tree_app<'ctx>(arena: &'ctx Bump, content: &str) -> (ReportContext<'ctx>, App<'ctx>) {
        use okane_core::report::BalanceTree;
        use okane_core::report::query::BalanceQuery;

        let (ctx, mut ledger) = process(arena, content);
        let balance = ledger
            .balance(&ctx, &BalanceQuery::default())
            .unwrap()
            .into_owned();
        let tree = BalanceTree::create(&ctx, balance).unwrap().into_nodes();
        let app = App::new("test".to_owned(), tree, template());
        (ctx, app)
    }

    fn row_labels(app: &App<'_>) -> Vec<String> {
        app.balance.rows
            .iter()
            .map(|r| r.label.to_owned())
            .collect()
    }

    fn full_names(app: &App<'_>) -> Vec<String> {
        app.balance.rows
            .iter()
            .map(|r| r.full_name().to_owned())
            .collect()
    }

    #[test]
    fn flat_mode_shows_only_posted_accounts() {
        let arena = Bump::new();
        let (_ctx, app) = tree_app(&arena, TREE_LEDGER);
        // Ancestor-only nodes (Assets, Assets:Bank, Expenses) have a zero own
        // amount and are hidden; every posted account is shown, full-name.
        assert_eq!(
            full_names(&app),
            [
                "Assets:Bank:Checking",
                "Assets:Cash",
                "Equity",
                "Expenses:Food",
            ]
        );
    }

    #[test]
    fn toggle_tree_shows_hierarchy_with_leaf_labels() {
        let arena = Bump::new();
        let (_ctx, mut app) = tree_app(&arena, TREE_LEDGER);
        // Flat rows 0/1 are Assets:Bank:Checking (10 USD) and Assets:Cash (5 USD).
        let checking = app.balance.rows[0].amount.clone();
        let cash = app.balance.rows[1].amount.clone();
        app.update(Message::ToggleTree);
        assert_eq!(app.balance.mode, DisplayMode::Tree);
        // Pre-order over the whole hierarchy, leaf labels, ancestors included.
        assert_eq!(
            full_names(&app),
            [
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
            row_labels(&app),
            [
                "Assets", "Bank", "Checking", "Cash", "Equity", "Expenses", "Food"
            ]
        );
        // Depth drives indentation: Assets is depth 1, Checking depth 3.
        assert_eq!(app.balance.rows[0].depth, 1);
        assert!(app.balance.rows[0].has_children);
        assert_eq!(app.balance.rows[2].depth, 3);
        assert!(!app.balance.rows[2].has_children);
        // Tree view shows subtree totals: Assets rolls up Checking + Cash.
        assert_eq!(app.balance.rows[0].amount, checking + cash);
        // Back to flat.
        app.update(Message::ToggleTree);
        assert_eq!(app.balance.mode, DisplayMode::Flat);
        assert_eq!(app.balance.rows.len(), 4);
    }

    #[test]
    fn toggle_fold_hides_selected_subtree() {
        let arena = Bump::new();
        let (_ctx, mut app) = tree_app(&arena, TREE_LEDGER);
        app.update(Message::ToggleTree);
        app.balance.nav.select(0); // Assets
        app.update(Message::ToggleFold);
        // Assets is folded: its Bank/Checking/Cash descendants disappear.
        assert_eq!(
            full_names(&app),
            ["Assets", "Equity", "Expenses", "Expenses:Food"]
        );
        assert!(app.balance.rows[0].folded);
        // Selection stays on the fold point.
        assert_eq!(app.balance.nav.table_state.selected(), Some(0));
        // Unfolding restores the descendants.
        app.update(Message::ToggleFold);
        assert!(!app.balance.rows[0].folded);
        assert_eq!(app.balance.rows.len(), 7);
    }

    #[test]
    fn toggle_fold_all_collapses_then_expands() {
        let arena = Bump::new();
        let (_ctx, mut app) = tree_app(&arena, TREE_LEDGER);
        app.update(Message::ToggleTree);
        app.update(Message::ToggleFoldAll);
        // Every foldable node collapses: only top-level rows remain.
        assert_eq!(full_names(&app), ["Assets", "Equity", "Expenses"]);
        app.update(Message::ToggleFoldAll);
        assert_eq!(app.balance.rows.len(), 7);
    }

    #[test]
    fn tree_open_register_drills_into_subtree() {
        let arena = Bump::new();
        let (_ctx, mut app) = tree_app(&arena, TREE_LEDGER);
        app.update(Message::ToggleTree);
        app.balance.nav.select(0); // Assets
        let cmd = app.update(Message::OpenRegister);
        assert_matches!(
            cmd,
            Some(Command::LoadRegister { scope: RegisterScope::Subtree(agg) }) if agg.as_str() == "Assets"
        );
    }

    #[test]
    fn flat_open_register_drills_into_single_account() {
        let arena = Bump::new();
        let (_ctx, mut app) = tree_app(&arena, TREE_LEDGER);
        app.balance.nav.select(1); // Assets:Cash
        let cmd = app.update(Message::OpenRegister);
        assert_matches!(
            cmd,
            Some(Command::LoadRegister { scope: RegisterScope::Single(account) }) if account.as_str() == "Assets:Cash"
        );
    }

    #[test]
    fn tree_state_survives_snapshot_restore() {
        let arena = Bump::new();
        let (_ctx, mut app) = tree_app(&arena, TREE_LEDGER);
        app.update(Message::ToggleTree);
        app.balance.nav.select(0); // Assets
        app.update(Message::ToggleFold); // fold Assets
        let snapshot = app.snapshot();

        let (_ctx2, mut app2) = tree_app(&arena, TREE_LEDGER);
        assert_matches!(app2.restore(&snapshot), None);
        assert_eq!(app2.balance.mode, DisplayMode::Tree);
        assert_eq!(
            full_names(&app2),
            ["Assets", "Equity", "Expenses", "Expenses:Food"]
        );
        assert!(app2.balance.rows[0].folded);
    }
}
