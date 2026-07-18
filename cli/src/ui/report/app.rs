//! UI application state.
//!
//! Follows The Elm Architecture: state lives in [`App`], all transitions go
//! through [`App::update`] driven by a [`Message`]. Key handling in
//! [`super::event`] translates raw `KeyEvent`s into messages based on the
//! currently active screen and overlay.

use std::cmp::{max, min};

use chrono::NaiveDate;
use okane_core::report::query::{Conversion, DateRange};
use okane_core::report::{Account, Amount};
use regex::RegexBuilder;

use crate::ui::table::TableNav;

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
        max(
            amount_line_count(&self.amount),
            amount_line_count(&self.total),
        )
    }
}

/// Number of lines an [`Amount`] would render as in a table.
fn amount_line_count(amount: &Amount<'_>) -> u16 {
    let n = amount.iter().count();
    n.clamp(1, u16::MAX as usize) as u16
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

/// A scroll request against a scrollable overlay body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDelta {
    Lines(i16),
    Pages(i16),
    Top,
    Bottom,
}

/// Body of the error modal: a full error report the user scrolls through.
///
/// The message is pre-split into display lines, and the renderer does not
/// re-wrap them (annotate-snippets output is column-aligned — soft wrapping
/// would move the carets away from what they point at). That keeps
/// `lines.len()` the exact rendered line count, so the scroll bound is
/// computable — and testable — without a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPopup {
    /// Modal title, e.g. `failed to load main.ledger`.
    pub title: String,
    /// The error report, one entry per display line.
    pub lines: Vec<String>,
    /// Index of the first visible line.
    pub scroll: u16,
    /// Last known height of the body. Updated each frame, the same way
    /// [`crate::ui::table::TableNav::viewport_height`] is.
    pub viewport_height: u16,
}

impl ErrorPopup {
    pub fn new(title: String, lines: Vec<String>) -> Self {
        Self {
            title,
            lines,
            scroll: 0,
            viewport_height: 0,
        }
    }

    /// Rows the body can scroll before its last line reaches the bottom of the
    /// viewport. Zero when everything already fits.
    fn max_scroll(&self) -> u16 {
        let lines = u16::try_from(self.lines.len()).unwrap_or(u16::MAX);
        lines.saturating_sub(max(self.viewport_height, 1))
    }

    /// Applies a scroll request, clamped to the scrollable range.
    pub fn scroll(&mut self, delta: ScrollDelta) {
        let page = i32::from(max(self.viewport_height, 1));
        let current = i32::from(self.scroll);
        let target = match delta {
            ScrollDelta::Lines(n) => current + i32::from(n),
            ScrollDelta::Pages(n) => current + i32::from(n) * page,
            ScrollDelta::Top => 0,
            ScrollDelta::Bottom => i32::from(self.max_scroll()),
        };
        self.scroll = target.clamp(0, i32::from(self.max_scroll())) as u16;
    }

    /// Re-clamps the offset after the viewport height changes (terminal resize).
    pub fn clamp(&mut self) {
        self.scroll = min(self.scroll, self.max_scroll());
    }
}

/// Modal overlay drawn on top of the current screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    /// "Quit? y/n" prompt shown when leaving the balance screen.
    QuitConfirm,
    /// A failure the user must acknowledge, shown in full.
    Error(ErrorPopup),
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
        account: Account<'ctx>,
    },
    /// Re-run the whole load/process pipeline and swap the data in.
    Reload,
}

/// Snapshot of register view that surives a reload, as part of [`UiSnapshot`].
#[derive(Debug, Clone)]
pub struct RegisterSnapshot {
    /// account filter for the registry.
    account: String,
    /// Selected cursor index.
    cursor: usize,
}

/// UI state that survives a reload, captured by [`App::snapshot`] and
/// re-applied by [`App::restore`]. Everything is plain owned data — arena
/// references would dangle across the reload's arena reset, so accounts
/// are kept by name and re-resolved against the rebuilt session.
#[derive(Debug, Clone)]
pub struct UiSnapshot {
    /// Carried over so page-up/down keeps working before the first frame.
    viewport_height: u16,
    /// Selected account on the balance screen, by name.
    selected_account: Option<String>,
    /// Snapshot of the register if it's register view.
    /// Extend this once [`Screen`] is more than 2 states.
    register: Option<RegisterSnapshot>,
    /// Active search intent; the matches are recomputed on restore.
    search: Option<SearchIntent>,
    last_search: String,
}

/// Application state for the TUI session.
#[derive(Debug)]
pub struct App<'ctx> {
    pub source_display: String,
    pub balance_rows: Vec<BalanceRow<'ctx>>,
    pub balance_nav: TableNav,
    pub screen: Screen<'ctx>,
    pub overlay: Option<Overlay>,
    /// Transient one-line notice shown in the footer. Cleared on the next key
    /// press. Failures worth reading in full go to [`Overlay::Error`] instead,
    /// which is dismissed explicitly.
    pub error_toast: Option<String>,
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
            error_toast: None,
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
            Message::Reload => return Some(Command::Reload),
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
    pub fn show_register(&mut self, account: Account<'ctx>, rows: Vec<RegisterRow<'ctx>>) {
        self.screen = Screen::Register(RegisterView::new(account, rows));
    }

    /// Like [`Self::show_register`], but restores a previous cursor position
    /// (clamped to the new row count) instead of jumping to the last entry.
    /// Used when re-entering the register after a reload.
    pub fn show_register_at(
        &mut self,
        account: Account<'ctx>,
        rows: Vec<RegisterRow<'ctx>>,
        index: usize,
    ) {
        let mut view = RegisterView::new(account, rows);
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
            viewport_height: self.balance_nav.viewport_height,
            selected_account: self
                .selected_balance_account()
                .map(|a| a.as_str().to_owned()),
            register: match &self.screen {
                Screen::Balance => None,
                Screen::Register(view) => Some(RegisterSnapshot {
                    account: view.account.as_str().to_owned(),
                    cursor: view.nav.table_state.selected().unwrap_or(0),
                }),
            },
            search: self.search.as_ref().map(|s| s.intent.clone()),
            last_search: self.last_search.clone(),
        }
    }

    /// Restores a [`UiSnapshot`] into this freshly-built `App`: the balance
    /// selection follows the previously selected account (or the closest one
    /// by name when it disappeared), and any active search is recomputed
    /// against the new rows.
    ///
    /// When the snapshot had the register screen open, returns
    /// `Some((account, index))` asking the caller to query that account's
    /// register and open it via [`Self::show_register_at`]. If the account
    /// no longer exists, stays on the balance screen (with a notice) and
    /// returns `None`.
    pub fn restore(&mut self, snapshot: &UiSnapshot) -> Option<(Account<'ctx>, usize)> {
        self.balance_nav.viewport_height = snapshot.viewport_height;
        if let Some(prev) = &snapshot.selected_account
            && let Some(idx) = restore_index(prev, &self.balance_rows)
        {
            self.balance_nav.select(idx);
        }

        self.last_search = snapshot.last_search.clone();
        if let Some(intent) = &snapshot.search {
            let mut intent = intent.clone();
            intent.origin = min(intent.origin, self.balance_rows.len().saturating_sub(1));
            let matches = SearchMatch::compute(&intent.input, &self.balance_rows);
            self.search = Some(Search { intent, matches });
        }

        let RegisterSnapshot{account, cursor} = snapshot.register.as_ref()?;
        match self
            .balance_rows
            .binary_search_by(|r| r.account.as_str().cmp(account.as_str()))
        {
            Ok(row) => Some((self.balance_rows[row].account, *cursor)),
            Err(_) => {
                self.error_toast = Some(format!(
                    "account {account} is gone after reload; back to balance"
                ));
                None
            }
        }
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

/// Row index to restore after a reload: the row of `prev_name` when it still
/// exists, otherwise the alphabetically closest row (insertion point, clamped
/// to the end). `None` when `rows` is empty.
///
/// Relies on `rows` being sorted by account name, which is the order
/// `Balance::into_vec` produces.
fn restore_index(prev_name: &str, rows: &[BalanceRow<'_>]) -> Option<usize> {
    let last = rows.len().checked_sub(1)?;
    let idx = rows
        .binary_search_by(|r| r.account.as_str().cmp(prev_name))
        .unwrap_or_else(|insertion| insertion);
    Some(min(idx, last))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use assert_matches::assert_matches;
    use bumpalo::Bump;
    use okane_core::report::ReportContext;
    use okane_core::{load, report};
    use rust_decimal_macros::dec;

    use crate::ui::table::TableNav;

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
    fn popup_scroll_clamps_at_top() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Lines(-1));
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_scroll_clamps_at_bottom() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Lines(100));
        assert_eq!(p.scroll, 6);
    }

    #[test]
    fn popup_scroll_pinned_when_body_fits() {
        let mut p = popup(5, 10);
        assert_eq!(p.max_scroll(), 0);
        p.scroll(ScrollDelta::Bottom);
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_page_scroll_uses_viewport_height() {
        let mut p = popup(100, 4);
        p.scroll(ScrollDelta::Pages(1));
        assert_eq!(p.scroll, 4);
        p.scroll(ScrollDelta::Pages(-1));
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_top_and_bottom_jump() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Bottom);
        assert_eq!(p.scroll, 6);
        p.scroll(ScrollDelta::Top);
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_scroll_without_viewport_does_not_panic() {
        // The first key can in principle arrive before a frame has been drawn;
        // an unknown viewport falls back to a single line per page.
        let mut p = popup(10, 0);
        p.scroll(ScrollDelta::Pages(1));
        assert_eq!(p.scroll, 1);
        p.scroll(ScrollDelta::Bottom);
        assert_eq!(p.scroll, 9);
    }

    #[test]
    fn popup_clamps_after_viewport_shrink() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Bottom);
        p.viewport_height = 10;
        p.clamp();
        assert_eq!(p.scroll, 0);
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

    /// Balance rows for `names` (must be sorted, matching `Balance::into_vec`
    /// order) resolved against an existing context.
    fn rows_of<'ctx>(ctx: &ReportContext<'ctx>, names: &[&str]) -> Vec<BalanceRow<'ctx>> {
        names
            .iter()
            .map(|n| BalanceRow {
                account: ctx.account(n).unwrap(),
                amount: Amount::zero(),
            })
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
        App::new("test".to_owned(), rows_of(ctx, names), template())
    }

    #[test]
    fn restore_follows_selected_account() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        app.balance_nav.select(2); // Expenses:Food
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
        app.balance_nav.select(2); // Expenses:Food
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
        app.balance_nav.viewport_height = 12;
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, ACCOUNTS);
        app.restore(&snapshot);
        assert_eq!(app.balance_nav.viewport_height, 12);
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
        assert_eq!(app.search.as_ref().unwrap().matched_rows(), [0, 1]);
        app.last_search = "salary".to_owned();
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, &["Assets:Bank", "Income:Salary"]);
        app.restore(&snapshot);
        let search = app.search.as_ref().unwrap();
        assert_eq!(search.intent.input, "assets");
        assert_eq!(search.matched_rows(), [0]);
        // Origin is clamped into the new row range.
        assert!(search.intent.origin < 2);
        assert_eq!(&app.last_search, "salary");
    }

    #[test]
    fn restore_requests_register_requery() {
        let arena = Bump::new();
        let (ctx, mut app) = make_balance_app(&arena, ACCOUNTS);
        let account = ctx.account("Assets:Cash").unwrap();
        let mut nav = TableNav::new(5);
        nav.select(3);
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav,
        });
        let snapshot = app.snapshot();

        let mut app = next_app(&ctx, ACCOUNTS);
        let got = app.restore(&snapshot);
        assert_matches!(got, Some((acc, 3)) if acc.as_str() == "Assets:Cash");
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
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
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
        app.show_register_at(account, rows.clone(), 10);
        let Screen::Register(view) = &app.screen else {
            panic!("expected register screen");
        };
        assert_eq!(view.nav.table_state.selected(), Some(2));

        app.show_register_at(account, rows, 1);
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
