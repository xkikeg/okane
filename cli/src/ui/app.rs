//! UI application state.
//!
//! Follows The Elm Architecture: state lives in [`App`], all transitions go
//! through [`App::update`] driven by a [`Message`]. Key handling in
//! [`super::event`] translates raw `KeyEvent`s into messages based on the
//! currently active screen and overlay.

use chrono::NaiveDate;
use okane_core::report::query::{Conversion, DateRange};
use okane_core::report::{Account, Amount};
use ratatui::widgets::TableState;

/// One row of the balance table.
///
/// Stores the typed account/amount values from the report layer so rendering
/// can reformat lazily under different display contexts (currency conversion,
/// commodity toggling, etc.) without rebuilding the row vector.
#[derive(Debug, Clone)]
pub struct BalanceRow<'ctx> {
    pub account: Account<'ctx>,
    pub amount: Amount<'ctx>,
}

impl BalanceRow<'_> {
    /// Number of rendered lines this row occupies (>= 1).
    ///
    /// One line per commodity, with a `0` placeholder line for empty balances.
    pub fn line_count(&self) -> u16 {
        amount_line_count(&self.amount)
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
        amount_line_count(&self.amount).max(amount_line_count(&self.total))
    }
}

/// Number of lines an [`Amount`] would render as in a table.
fn amount_line_count(amount: &Amount<'_>) -> u16 {
    let n = amount.iter().count();
    n.max(1).min(u16::MAX as usize) as u16
}

/// Pure scroll/selection state for a table.
///
/// Lives separately from row data so the navigation math can be tested
/// without constructing a `'ctx`-bound `App` or a real `ReportContext`.
#[derive(Debug, Default)]
pub struct TableNav {
    pub table_state: TableState,
    pub row_count: usize,
    /// Last known viewport height for the table body. Updated each frame and
    /// used to size page-up/page-down jumps.
    pub viewport_height: u16,
}

impl TableNav {
    pub fn new(row_count: usize) -> Self {
        let mut table_state = TableState::default();
        if row_count > 0 {
            table_state.select(Some(0));
        }
        Self {
            table_state,
            row_count,
            viewport_height: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    fn last_index(&self) -> Option<usize> {
        self.row_count.checked_sub(1)
    }

    /// Moves the selection by `delta` rows, clamping to the row range.
    pub fn move_selection(&mut self, delta: isize) {
        let Some(last) = self.last_index() else {
            return;
        };
        let current = self.table_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, last as isize);
        self.table_state.select(Some(next as usize));
    }

    /// Page size — at least 1 row, falls back to a sensible default if the
    /// viewport height has not been observed yet.
    pub fn page_size(&self) -> usize {
        self.viewport_height.max(1) as usize
    }

    pub fn select_first(&mut self) {
        if !self.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if let Some(last) = self.last_index() {
            self.table_state.select(Some(last));
        }
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
    pub account: String,
    pub rows: Vec<RegisterRow<'ctx>>,
    pub nav: TableNav,
}

impl<'ctx> RegisterView<'ctx> {
    pub fn new(account: String, rows: Vec<RegisterRow<'ctx>>) -> Self {
        let mut nav = TableNav::new(rows.len());
        // Most recent entry is the most useful starting point.
        nav.select_last();
        Self { account, rows, nav }
    }
}

/// Top-level screen the user is currently looking at.
#[derive(Debug)]
pub enum Screen<'ctx> {
    Balance,
    Register(RegisterView<'ctx>),
}

/// Modal overlay drawn on top of the current screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    /// "Quit? y/n" prompt shown when leaving the balance screen.
    QuitConfirm,
}

/// Messages that drive state transitions (Elm-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    SelectFirst,
    SelectLast,
    /// User asked to drill into the selected balance row.
    OpenRegister,
    /// Leave the register view and go back to balance.
    LeaveRegister,
    /// User asked to quit from balance — show the confirmation overlay.
    RequestQuit,
    /// Confirm quit from the overlay.
    ConfirmQuit,
    /// Dismiss the current overlay.
    DismissOverlay,
    /// Unconditional quit (Ctrl-C).
    QuitImmediate,
}

/// Effect requested by [`App::update`] that requires resources the pure
/// state machine does not own (here: `&mut Ledger` to compute a register).
#[derive(Debug, Clone)]
pub enum Command {
    LoadRegister { account: String },
}

/// Application state for the TUI session.
#[derive(Debug)]
pub struct App<'ctx> {
    pub source_display: String,
    pub balance_rows: Vec<BalanceRow<'ctx>>,
    pub balance_nav: TableNav,
    pub screen: Screen<'ctx>,
    pub overlay: Option<Overlay>,
    pub register_template: RegisterQueryTemplate<'ctx>,
    pub should_quit: bool,
}

impl<'ctx> App<'ctx> {
    pub fn new(
        source_display: String,
        balance_rows: Vec<BalanceRow<'ctx>>,
        register_template: RegisterQueryTemplate<'ctx>,
    ) -> Self {
        let balance_nav = TableNav::new(balance_rows.len());
        Self {
            source_display,
            balance_rows,
            balance_nav,
            screen: Screen::Balance,
            overlay: None,
            register_template,
            should_quit: false,
        }
    }

    /// The currently-selected balance account, if any.
    pub fn selected_balance_account(&self) -> Option<&str> {
        let idx = self.balance_nav.table_state.selected()?;
        self.balance_rows.get(idx).map(|r| r.account.as_str())
    }

    /// Mutable handle to whichever nav drives the currently visible table.
    fn active_nav_mut(&mut self) -> &mut TableNav {
        match &mut self.screen {
            Screen::Balance => &mut self.balance_nav,
            Screen::Register(view) => &mut view.nav,
        }
    }

    /// Applies a message; optionally returns a [`Command`] for the event
    /// loop to execute (the only impure step in this flow).
    pub fn update(&mut self, msg: Message) -> Option<Command> {
        // QuitImmediate is honored regardless of overlay/screen.
        if matches!(msg, Message::QuitImmediate) {
            self.should_quit = true;
            return None;
        }

        if self.overlay.is_some() {
            match msg {
                Message::ConfirmQuit => self.should_quit = true,
                Message::DismissOverlay => self.overlay = None,
                // Ignore other input while a modal is up.
                _ => {}
            }
            return None;
        }

        match msg {
            Message::MoveUp => self.active_nav_mut().move_selection(-1),
            Message::MoveDown => self.active_nav_mut().move_selection(1),
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
                    && let Some(account) = self.selected_balance_account()
                {
                    return Some(Command::LoadRegister {
                        account: account.to_owned(),
                    });
                }
            }
            Message::LeaveRegister => {
                if matches!(self.screen, Screen::Register(_)) {
                    self.screen = Screen::Balance;
                }
            }
            Message::RequestQuit => {
                if matches!(self.screen, Screen::Balance) {
                    self.overlay = Some(Overlay::QuitConfirm);
                }
            }
            // Already handled above.
            Message::QuitImmediate | Message::ConfirmQuit | Message::DismissOverlay => {}
        }
        None
    }

    /// Called by the event loop once a [`Command::LoadRegister`] has been
    /// fulfilled.
    pub fn show_register(&mut self, account: String, rows: Vec<RegisterRow<'ctx>>) {
        self.screen = Screen::Register(RegisterView::new(account, rows));
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use okane_core::report::ReportContext;
    use rust_decimal_macros::dec;

    use super::*;

    fn nav(n: usize) -> TableNav {
        TableNav::new(n)
    }

    fn template<'ctx>() -> RegisterQueryTemplate<'ctx> {
        RegisterQueryTemplate {
            conversion: None,
            date_range: DateRange::default(),
        }
    }

    /// Build an `App` with no balance rows — sufficient for testing the
    /// pure state machine. (Constructing a `BalanceRow` requires an
    /// `Account<'ctx>`, whose interner has no public constructor outside
    /// `okane_core`, so we side-step it.)
    fn app_no_rows<'ctx>() -> App<'ctx> {
        App::new("test".to_owned(), Vec::new(), template())
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
    fn empty_nav_has_no_selection() {
        let n = nav(0);
        assert!(n.is_empty());
        assert_eq!(n.table_state.selected(), None);
    }

    #[test]
    fn move_selection_clamps_to_bounds() {
        let mut n = nav(3);
        assert_eq!(n.table_state.selected(), Some(0));

        n.move_selection(-1);
        assert_eq!(n.table_state.selected(), Some(0));

        n.move_selection(1);
        assert_eq!(n.table_state.selected(), Some(1));

        n.move_selection(100);
        assert_eq!(n.table_state.selected(), Some(2));

        n.move_selection(-100);
        assert_eq!(n.table_state.selected(), Some(0));
    }

    #[test]
    fn select_first_and_last() {
        let mut n = nav(5);
        n.select_last();
        assert_eq!(n.table_state.selected(), Some(4));
        n.select_first();
        assert_eq!(n.table_state.selected(), Some(0));
    }

    #[test]
    fn select_first_or_last_on_empty_is_noop() {
        let mut n = nav(0);
        n.select_last();
        assert_eq!(n.table_state.selected(), None);
        n.select_first();
        assert_eq!(n.table_state.selected(), None);
    }

    #[test]
    fn page_size_defaults_to_one_when_unset() {
        let n = nav(10);
        assert_eq!(n.page_size(), 1);
    }

    #[test]
    fn page_size_uses_viewport_height() {
        let mut n = nav(10);
        n.viewport_height = 20;
        assert_eq!(n.page_size(), 20);
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
        app.balance_nav = TableNav::new(3);
        app.update(Message::RequestQuit);
        app.update(Message::MoveDown);
        assert_eq!(app.balance_nav.table_state.selected(), Some(0));
    }

    #[test]
    fn leave_register_returns_to_balance() {
        let mut app = app_no_rows();
        // Bypass show_register's RegisterView::new — it just needs *some*
        // register screen state to flip the enum variant.
        app.screen = Screen::Register(RegisterView {
            account: "Assets:Cash".to_owned(),
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        app.update(Message::LeaveRegister);
        assert!(matches!(app.screen, Screen::Balance));
    }

    #[test]
    fn request_quit_from_register_does_not_open_overlay() {
        let mut app = app_no_rows();
        app.screen = Screen::Register(RegisterView {
            account: "Assets:Cash".to_owned(),
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        assert!(app.update(Message::RequestQuit).is_none());
        // From register, q/Esc leaves to balance (mapped at the event layer)
        // rather than opening the quit overlay.
        assert_eq!(app.overlay, None);
    }
}
