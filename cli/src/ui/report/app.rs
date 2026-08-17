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

use okane_core::report::BalanceTreeNode;

use super::balance::{BalanceAction, BalanceMessage, BalanceSnapshot, BalanceView};
use super::help;
use super::overlay::{Overlay, ScrollDelta};
use super::register::{
    RegisterAction, RegisterMessage, RegisterQueryTemplate, RegisterRow, RegisterScope,
    RegisterSnapshot, RegisterView,
};

/// Top-level screen the user is currently looking at.
#[derive(Debug)]
pub enum Screen<'ctx> {
    Balance,
    Register(RegisterView<'ctx>),
}

/// Messages that drive state transitions (Elm-style).
///
/// [`App`] owns only the cross-screen messages; per-screen behavior is carried
/// by [`BalanceMessage`] and [`RegisterMessage`] and routed to the focused
/// component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// A message routed to the balance screen.
    Balance(BalanceMessage),
    /// A message routed to the active register screen.
    Register(RegisterMessage),
    /// User asked to quit from balance — show the confirmation overlay.
    RequestQuit,
    /// Confirm quit from the overlay.
    ConfirmQuit,
    /// Dismiss the current overlay.
    DismissOverlay,
    /// Show the key help for the focused screen (`?` / `F1`).
    ShowHelp,
    /// Scroll the body of a scrollable overlay.
    OverlayScroll(ScrollDelta),
    /// Unconditional quit (Ctrl-C).
    QuitImmediate,
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
                    if let Some(popup) = self.overlay.as_mut().and_then(Overlay::scrollable_mut) {
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
            // Route to the focused component; translate its action into the
            // cross-screen effect App owns.
            Message::Balance(balance_msg) => {
                if matches!(self.screen, Screen::Balance)
                    && let Some(action) = self.balance.update(balance_msg)
                {
                    return Some(match action {
                        BalanceAction::OpenRegister { scope } => Command::LoadRegister { scope },
                    });
                }
            }
            Message::Register(register_msg) => {
                if let Screen::Register(view) = &mut self.screen
                    && let Some(action) = view.update(register_msg)
                {
                    match action {
                        RegisterAction::Leave => self.screen = Screen::Balance,
                    }
                }
            }
            Message::RequestQuit => {
                if matches!(self.screen, Screen::Balance) {
                    self.overlay = Some(Overlay::QuitConfirm);
                }
            }
            Message::Reload => return Some(Command::Reload),
            Message::ShowHelp => self.overlay = Some(Overlay::Help(help::popup(&self.screen))),
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
    /// (an entry index, clamped to the new entry count) instead of jumping to
    /// the last entry. Used when re-entering the register after a reload.
    pub fn show_register_at(
        &mut self,
        scope: RegisterScope<'ctx>,
        rows: Vec<RegisterRow<'ctx>>,
        index: usize,
    ) {
        let mut view = RegisterView::new(scope, rows);
        if let Some(last) = view.nav.item_count().checked_sub(1) {
            view.nav.select_item(min(index, last));
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
    use okane_core::report::{Account, Amount, ReportContext};

    use crate::ui::table::{NavCommand, TableNav};

    use super::super::balance::BalanceRow;
    use super::super::overlay::TextPopup;
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

    /// The selected *account* (an index into `app.balance.rows`), which is
    /// what every test below means — the table itself has one row per
    /// commodity line.
    fn selected(app: &App<'_>) -> Option<usize> {
        app.balance.nav.selected_item()
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
        assert!(
            app.update(Message::Balance(BalanceMessage::OpenRegister))
                .is_none()
        );
        assert!(matches!(app.screen, Screen::Balance));
    }

    #[test]
    fn nav_messages_ignored_while_overlay_visible() {
        let mut app = app_no_rows();
        // Pretend there are rows to move through by poking the nav directly.
        app.balance.nav = TableNav::new(3);
        app.update(Message::RequestQuit);
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
        assert_eq!(app.balance.nav.selected_row(), 0);
    }

    fn popup(lines: usize, viewport_height: u16) -> TextPopup {
        let mut popup = TextPopup::new(
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
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
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

    /// The help is built for the screen it was opened from, so the register's
    /// lists the register's keys.
    #[test]
    fn show_help_opens_the_help_of_the_focused_screen() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let mut app = app_no_rows();

        app.update(Message::ShowHelp);
        assert_matches!(&app.overlay, Some(Overlay::Help(p)) if p.title.contains("balance"));

        app.update(Message::DismissOverlay);
        app.screen = register_screen(account, TableNav::new(0));
        app.update(Message::ShowHelp);
        assert_matches!(&app.overlay, Some(Overlay::Help(p)) if p.title.contains("register"));
    }

    #[test]
    fn help_scrolls_on_overlay_scroll() {
        let mut app = app_no_rows();
        app.update(Message::ShowHelp);
        let Some(Overlay::Help(popup)) = app.overlay.as_mut() else {
            panic!("expected the help overlay");
        };
        popup.viewport_height = 4;
        app.update(Message::OverlayScroll(ScrollDelta::Bottom));
        assert_matches!(&app.overlay, Some(Overlay::Help(p)) if p.scroll > 0);
    }

    /// The help is a read-and-leave page: while it is up, the keys underneath
    /// it do nothing.
    #[test]
    fn help_swallows_the_screen_underneath() {
        let mut app = app_no_rows();
        app.balance.nav = TableNav::new(3);
        app.update(Message::ShowHelp);
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
        assert_eq!(app.balance.nav.selected_row(), 0);
        assert_matches!(app.overlay, Some(Overlay::Help(_)));
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
    fn reload_message_produces_command() {
        let mut app = app_no_rows();
        assert_matches!(app.update(Message::Reload), Some(Command::Reload));
    }

    #[test]
    fn any_key_clears_error_notice() {
        let mut app = app_no_rows();
        app.error_toast = Some("boom".to_owned());
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
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
        app.balance.nav.select_item(2); // Expenses:Food
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
        app.balance.nav.select_item(2); // Expenses:Food
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
        app.update(Message::Balance(BalanceMessage::StartModalSearch));
        for c in "assets".chars() {
            app.update(Message::Balance(BalanceMessage::SearchPush(c)));
        }
        app.update(Message::Balance(BalanceMessage::SearchSubmit)); // fixed
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
        nav.select_item(3);
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
        assert_eq!(view.nav.selected_item(), Some(2));

        app.show_register_at(scope, rows, 1);
        let Screen::Register(view) = &app.screen else {
            panic!("expected register screen");
        };
        assert_eq!(view.nav.selected_item(), Some(1));
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
}
