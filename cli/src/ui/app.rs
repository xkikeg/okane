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
use regex::RegexBuilder;

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

    /// Selects an explicit row index, ignored when out of range.
    pub fn select(&mut self, index: usize) {
        if index < self.row_count {
            self.table_state.select(Some(index));
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
    pub account: Account<'ctx>,
    pub rows: Vec<RegisterRow<'ctx>>,
    pub nav: TableNav,
}

impl<'ctx> RegisterView<'ctx> {
    pub fn new(account: Account<'ctx>, rows: Vec<RegisterRow<'ctx>>) -> Self {
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

/// Phase of the modal (`/`) account search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPhase {
    /// Pattern is being typed; matches recompute on every keystroke.
    Incremental,
    /// Pattern is frozen; `n`/`N` jump between matches.
    Fixed,
}

/// Direction an interactive search last moved in. Determines which way fresh
/// input jumps (forward `C-s` vs backward `C-r`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Interaction style of an account search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Modal `/` search: incremental editing, then a frozen `n`/`N` phase.
    Modal(SearchPhase),
    /// Interactive `C-s`/`C-r` search (i-search): editing is always live.
    Interactive,
}

/// What the user is searching for and how — pure intent, no computed state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchIntent {
    pub mode: SearchMode,
    /// Direction of the search.
    /// Currently Modal search is only provided with forward,
    /// but implementing backward won't be hard.
    pub dir: SearchDirection,
    /// Raw pattern as typed (without the leading `/` or `I-search:` prompt).
    pub input: String,
    /// Set when `C-s`/`C-r` was pressed on an empty interactive pattern but no
    /// previous search text exists; drives the `[no previous search text]`
    /// notice. Cleared as soon as the pattern changes.
    pub no_previous: bool,
    /// Balance selection when search started; restored on cancel/abort.
    pub origin: usize,
}

/// Computed set of balance-row indices that matched the search pattern.
/// Newtype so we can attach match-specific methods.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SearchMatch(Vec<usize>);

impl From<Vec<usize>> for SearchMatch {
    fn from(v: Vec<usize>) -> Self {
        Self(v)
    }
}

impl SearchMatch {
    fn rows(&self) -> &[usize] {
        &self.0
    }

    /// Returns true if it contains the row
    pub fn contains_row(&self, i: usize) -> bool {
        self.0.binary_search(&i).is_ok()
    }

    /// First match at-or-after/before `pos` depending on `dir`, wrapping around.
    /// Stays on `pos` if it is already a match. Returns `None` when empty.
    pub fn first_match(&self, pos: usize, dir: SearchDirection) -> Option<usize> {
        let rows = &self.0;
        if rows.is_empty() {
            return None;
        }
        let len = rows.len();
        let idx = match (rows.binary_search(&pos), dir) {
            (Ok(i), _) => i,
            (Err(i), SearchDirection::Forward) => i % len,
            (Err(i), SearchDirection::Backward) => (i + len - 1) % len,
        };
        Some(rows[idx])
    }

    /// Computes matching row indices for `input` as a case-insensitive regex.
    /// Returns `None` for empty input, `Err` for an invalid pattern.
    pub fn compute(input: &str, rows: &[BalanceRow<'_>]) -> Option<Result<Self, regex::Error>> {
        if input.is_empty() {
            return None;
        }
        Some(
            RegexBuilder::new(input)
                .case_insensitive(true)
                .build()
                .map(|re| {
                    Self(
                        rows.iter()
                            .enumerate()
                            .filter(|(_, row)| re.is_match(row.account.as_str()))
                            .map(|(i, _)| i)
                            .collect(),
                    )
                }),
        )
    }

    /// Row index of the next/previous match relative to `current` (wrapping).
    /// None if empty.
    pub fn step(&self, current: usize, dir: SearchDirection) -> Option<usize> {
        let rows = &self.0;
        if rows.is_empty() {
            return None;
        }
        let len = rows.len();
        let next_idx = match (rows.binary_search(&current), dir) {
            // `current` is a match: step one slot in the requested direction.
            (Ok(i), SearchDirection::Forward) => (i + 1) % len,
            (Ok(i), SearchDirection::Backward) => (i + len - 1) % len,
            // `current` is between matches: `i` is the insertion point, i.e. the
            // first match after `current` (mod len for the wrap).
            (Err(i), SearchDirection::Forward) => i % len,
            (Err(i), SearchDirection::Backward) => (i + len - 1) % len,
        };
        Some(rows[next_idx])
    }
}

/// Account-name search state on the balance screen.
///
/// Not `PartialEq` because `regex::Error` doesn't implement it — tests inspect
/// the individual fields.
#[derive(Debug)]
pub struct Search {
    pub intent: SearchIntent,
    /// `None` when `input` is empty; `Ok` with matching row indices; `Err` when
    /// the pattern fails to compile as a regex.
    pub matches: Option<Result<SearchMatch, regex::Error>>,
}

impl Search {
    pub(super) fn err(&self) -> Option<&regex::Error> {
        self.matches.as_ref()?.as_ref().err()
    }
    pub(super) fn matched_rows(&self) -> &[usize] {
        self.matches
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map_or(&[][..], |m| m.rows())
    }
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
}

/// Effect requested by [`App::update`] that requires resources the pure
/// state machine does not own (here: `&mut Ledger` to compute a register).
#[derive(Debug, Clone, Copy)]
pub enum Command<'ctx> {
    LoadRegister { account: Account<'ctx> },
}

/// Application state for the TUI session.
#[derive(Debug)]
pub struct App<'ctx> {
    pub source_display: String,
    pub balance_rows: Vec<BalanceRow<'ctx>>,
    pub balance_nav: TableNav,
    pub screen: Screen<'ctx>,
    pub overlay: Option<Overlay>,
    /// Active account search on the balance screen, if any.
    pub search: Option<Search>,
    /// Most recently used search pattern, recalled by an empty interactive
    /// search via `C-s`/`C-r`. Shared across modal and interactive searches.
    /// Not `Option` because empty string can represent empty state.
    pub last_search: String,
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
            search: None,
            last_search: String::new(),
            register_template,
            should_quit: false,
        }
    }

    /// The currently-selected balance account, if any.
    pub fn selected_balance_account(&self) -> Option<Account<'ctx>> {
        let idx = self.balance_nav.table_state.selected()?;
        self.balance_rows.get(idx).map(|r| r.account)
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
    pub fn update(&mut self, msg: Message) -> Option<Command<'ctx>> {
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
            Message::MoveUp => {
                self.end_interactive_search();
                self.active_nav_mut().move_selection(-1);
            }
            Message::MoveDown => {
                self.end_interactive_search();
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
                    && let Some(account) = self.selected_balance_account()
                {
                    // An interactive search drills in like the normal view:
                    // end the search, keeping the cursor on the chosen account.
                    self.end_interactive_search();
                    return Some(Command::LoadRegister { account });
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
            Message::StartModalSearch => self.start_search(
                SearchMode::Modal(SearchPhase::Incremental),
                SearchDirection::Forward,
            ),
            Message::StartISearch(dir) => self.start_search(SearchMode::Interactive, dir),
            Message::SearchPush(c) => {
                if let Some(search) = self.search.as_mut() {
                    search.intent.input.push(c);
                    search.intent.no_previous = false;
                }
                self.recompute_search();
            }
            Message::SearchPop => {
                if let Some(search) = self.search.as_mut() {
                    search.intent.input.pop();
                    search.intent.no_previous = false;
                }
                self.recompute_search();
            }
            Message::SearchSubmit => match &self.search {
                // If empty pattern submitted, simply exists the search mode.
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
            },
            Message::SearchCancel => {
                if let Some(search) = self.search.take() {
                    // on cancel, search query won't be saved.
                    self.balance_nav.select(search.intent.origin);
                }
            }
            Message::SearchClose => {
                self.search = None;
            }
            Message::SearchNext => self.search_or_recall(SearchDirection::Forward),
            Message::SearchPrev => self.search_or_recall(SearchDirection::Backward),
            // Already handled above.
            Message::QuitImmediate | Message::ConfirmQuit | Message::DismissOverlay => {}
        }
        None
    }

    /// Called by the event loop once a [`Command::LoadRegister`] has been
    /// fulfilled.
    pub fn show_register(&mut self, account: Account<'ctx>, rows: Vec<RegisterRow<'ctx>>) {
        self.screen = Screen::Register(RegisterView::new(account, rows));
    }

    /// Opens a search of the given style, recording the current selection as
    /// the origin. No-op off the balance screen or when one is already open.
    fn start_search(&mut self, mode: SearchMode, dir: SearchDirection) {
        if !matches!(self.screen, Screen::Balance) && self.search.is_none() {
            return;
        }
        let origin = self.balance_nav.table_state.selected().unwrap_or(0);
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
        let current = self.balance_nav.table_state.selected().unwrap_or(0);
        let Some(next) = m.step(current, search.intent.dir) else {
            return;
        };
        self.balance_nav.select(next);
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
            SearchMode::Interactive => self.balance_nav.table_state.selected().unwrap_or(origin),
        };
        let matches = SearchMatch::compute(&intent.input, &self.balance_rows);
        let jump = match &matches {
            Some(Ok(m)) => m.first_match(reference, intent.dir),
            _ => None,
        };
        search.matches = matches;
        if let Some(idx) = jump {
            self.balance_nav.select(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use assert_matches::assert_matches;
    use bumpalo::Bump;
    use okane_core::report::ReportContext;
    use okane_core::{load, report};
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

    /// Process a trivial ledger and return the context + a resolved account.
    fn make_account<'ctx>(
        arena: &'ctx Bump,
        account_name: &str,
    ) -> (ReportContext<'ctx>, Account<'ctx>) {
        let content = format!("2024/01/01 Init\n    {account_name}    100 USD\n    Equity\n");
        let mut map = HashMap::new();
        map.insert(PathBuf::from("test.ledger"), content.into_bytes());
        let loader = load::Loader::new(
            PathBuf::from("test.ledger"),
            load::FakeFileSystem::from(map),
        );
        let mut ctx = ReportContext::new(arena);
        let _ = report::process(&mut ctx, loader, &report::ProcessOptions::default()).unwrap();
        let account = ctx.account(account_name).unwrap();
        (ctx, account)
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
        let mut map = HashMap::new();
        map.insert(PathBuf::from("test.ledger"), content.into_bytes());
        let loader = load::Loader::new(
            PathBuf::from("test.ledger"),
            load::FakeFileSystem::from(map),
        );
        let mut ctx = ReportContext::new(arena);
        let _ = report::process(&mut ctx, loader, &report::ProcessOptions::default()).unwrap();
        let rows: Vec<BalanceRow> = names
            .iter()
            .map(|n| BalanceRow {
                account: ctx.account(n).unwrap(),
                amount: Amount::zero(),
            })
            .collect();
        let app = App::new("test".to_owned(), rows, template());
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
        app.balance_nav.table_state.selected()
    }

    #[test]
    fn step_match_next_and_prev_wrap() {
        let m = SearchMatch::from(vec![2usize, 5, 8]);
        // From a match.
        assert_eq!(m.step(5, SearchDirection::Forward), Some(8));
        assert_eq!(m.step(8, SearchDirection::Forward), Some(2)); // wrap forward
        assert_eq!(m.step(2, SearchDirection::Backward), Some(8)); // wrap backward
        assert_eq!(m.step(5, SearchDirection::Backward), Some(2));
        // From a non-match position.
        assert_eq!(m.step(4, SearchDirection::Forward), Some(5)); // first after 4
        assert_eq!(m.step(4, SearchDirection::Backward), Some(2)); // last before 4
        assert_eq!(m.step(0, SearchDirection::Backward), Some(8)); // before all, prev wraps
        assert_eq!(m.step(9, SearchDirection::Forward), Some(2)); // after all, next wraps
    }

    #[test]
    fn compute_matches_classifies_input() {
        let rows: &[BalanceRow<'_>] = &[];
        assert_matches!(SearchMatch::compute("", rows), None);
        assert_matches!(SearchMatch::compute("assets", rows), Some(Ok(_)));
        assert_matches!(SearchMatch::compute("[", rows), Some(Err(_)));
    }

    #[test]
    fn start_search_records_origin() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::MoveDown);
        app.update(Message::MoveDown);
        assert_eq!(selected(&app), Some(2));
        app.update(Message::StartModalSearch);
        let search = app.search.as_ref().expect("search active");
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
        let search = app.search.as_ref().unwrap();
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
        assert_eq!(app.search.as_ref().unwrap().matched_rows(), [0, 1]);
        assert_eq!(selected(&app), Some(0));
    }

    #[test]
    fn incremental_invalid_regex_sets_error() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        app.update(Message::SearchPush('['));
        let search = app.search.as_ref().unwrap();
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
        assert_eq!(app.search.as_ref().unwrap().matched_rows(), [1]);
        // Backspace down to "ca" — matches "Assets:Cash" and "Liabilities:Card".
        app.update(Message::SearchPop);
        app.update(Message::SearchPop);
        assert_eq!(app.search.as_ref().unwrap().matched_rows(), [1, 4]);
    }

    #[test]
    fn submit_empty_exits_search() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        app.update(Message::SearchSubmit);
        assert!(app.search.is_none());
    }

    #[test]
    fn submit_nonempty_enters_fixed_phase() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.update(Message::StartModalSearch);
        app.update(Message::SearchPush('a'));
        app.update(Message::SearchSubmit);
        assert_eq!(
            app.search.as_ref().unwrap().intent.mode,
            SearchMode::Modal(SearchPhase::Fixed)
        );
    }

    #[test]
    fn isearch_forward_jumps_and_repeats() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);

        app.update(Message::StartISearch(SearchDirection::Forward));

        let search = app.search.as_ref().unwrap();
        assert_eq!(search.intent.mode, SearchMode::Interactive);
        assert_eq!(search.intent.dir, SearchDirection::Forward);

        for c in "assets".chars() {
            app.update(Message::SearchPush(c));
        }

        // First forward match at-or-after origin 0.
        assert_eq!(app.search.as_ref().unwrap().matched_rows(), [0, 1]);
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
        assert!(app.search.is_none());
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
        assert_eq!(&app.last_search, "salary");

        // A fresh interactive search with an empty pattern recalls it on C-s.
        app.update(Message::StartISearch(SearchDirection::Forward));
        app.update(Message::SearchNext);
        let search = app.search.as_ref().unwrap();
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
        let search = app.search.as_ref().unwrap();
        assert!(search.intent.no_previous);
        assert!(search.intent.input.is_empty());
        // Typing clears the notice and resumes a normal search.
        app.update(Message::SearchPush('a'));
        assert!(!app.search.as_ref().unwrap().intent.no_previous);
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
        assert!(app.search.is_none());
        assert_eq!(selected(&app), Some(1));
        // The pattern is remembered for later recall.
        assert_eq!(&app.last_search, "assets");
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
        assert!(app.search.is_none());
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
        assert!(app.search.is_some());
    }

    #[test]
    fn isearch_recall_backward_sets_direction() {
        let arena = Bump::new();
        let (_ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.last_search = "assets".to_owned();
        app.update(Message::SelectLast); // origin 4
        app.update(Message::StartISearch(SearchDirection::Forward));
        // C-r on empty: recall + search backward from origin → last match (1).
        app.update(Message::SearchPrev);
        let search = app.search.as_ref().unwrap();
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
        assert!(app.search.is_none());
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
        assert!(app.search.is_none());
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
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let mut app = app_no_rows();
        // Bypass show_register's RegisterView::new — it just needs *some*
        // register screen state to flip the enum variant.
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        app.update(Message::LeaveRegister);
        assert!(matches!(app.screen, Screen::Balance));
    }

    #[test]
    fn request_quit_from_register_does_not_open_overlay() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let mut app = app_no_rows();
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        assert!(app.update(Message::RequestQuit).is_none());
        // From register, q/Esc leaves to balance (mapped at the event layer)
        // rather than opening the quit overlay.
        assert_eq!(app.overlay, None);
    }
}
