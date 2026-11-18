//! Search shared by the balance and register screens.
//!
//! Pure data and matching logic: [`SearchIntent`] captures *what* the user is
//! searching for and how, [`SearchMatch`] holds the computed row indices, and
//! [`Search`] pairs the two. [`SearchState`] is the component a screen embeds:
//! it owns the active search plus that screen's own search history, maps keys
//! to [`SearchMessage`]s, and moves the screen's [`TableNav`] selection over
//! any rows implementing [`Searchable`].

use std::borrow::Cow;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use regex::{Regex, RegexBuilder};

use crate::ui::keys::is_ctrl;
use crate::ui::migemo::{Migemo, MigemoError};
use crate::ui::table::TableNav;

/// How the text typed in the search bar becomes the regex the rows are matched
/// against.
///
/// [`Self::Plain`] is what a session without `--migemo` uses: the pattern is
/// the regex, exactly as typed. [`Self::Migemo`] runs it through a migemo
/// process first, so `ginkou` matches the accounts written 銀行 — the typed
/// text is still in migemo's output as one alternative, so plain ASCII
/// searching keeps working.
#[derive(Debug, Default)]
pub enum Translator {
    #[default]
    Plain,
    Migemo(Migemo),
}

impl Translator {
    /// The regex `input` stands for under this translation.
    fn to_regex<'a>(&self, input: &'a str) -> Result<Cow<'a, str>, SearchError> {
        match self {
            Translator::Plain => Ok(Cow::Borrowed(input)),
            Translator::Migemo(migemo) => Ok(Cow::Owned(migemo.query(input)?)),
        }
    }
}

/// Why the typed pattern produced no matches to show.
#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("invalid regex: {0}")]
    Regex(#[from] regex::Error),
    #[error("migemo failed: {0}")]
    Migemo(#[from] MigemoError),
}

impl SearchError {
    /// Short tag for the search bar, which has one line to say what went wrong.
    pub(super) fn label(&self) -> &'static str {
        match self {
            SearchError::Regex(_) => "[invalid regex]",
            SearchError::Migemo(err) => err.label(),
        }
    }
}

/// A table row the search can match against.
pub trait Searchable {
    /// Whether any of the row's searchable fields matches `re`.
    fn is_match(&self, re: &Regex) -> bool;
}

/// Phase of the modal (`/`) search.
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

/// Interaction style of a search.
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
    /// Selected item when search started; restored on cancel/abort.
    pub origin: usize,
}

/// Computed set of row indices that matched the search pattern.
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

    /// Computes matching row indices for `input`, translated by `translator`
    /// and compiled as a case-insensitive regex. Returns `None` for empty
    /// input, `Err` for a pattern that cannot be built.
    pub fn compute<T: Searchable>(
        input: &str,
        rows: &[T],
        translator: &Translator,
    ) -> Option<Result<Self, SearchError>> {
        if input.is_empty() {
            return None;
        }
        Some(Self::compute_nonempty(input, rows, translator))
    }

    fn compute_nonempty<T: Searchable>(
        input: &str,
        rows: &[T],
        translator: &Translator,
    ) -> Result<Self, SearchError> {
        let pattern = translator.to_regex(input)?;
        let re = RegexBuilder::new(&pattern).case_insensitive(true).build()?;
        Ok(Self(
            rows.iter()
                .enumerate()
                .filter(|(_, row)| row.is_match(&re))
                .map(|(i, _)| i)
                .collect(),
        ))
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

/// An active search: what is searched for, and what it matched.
///
/// Not `PartialEq` because [`SearchError`] doesn't implement it — tests inspect
/// the individual fields.
#[derive(Debug)]
pub struct Search {
    pub intent: SearchIntent,
    /// `None` when `input` is empty; `Ok` with matching row indices; `Err` when
    /// the pattern cannot be turned into a regex.
    pub matches: Option<Result<SearchMatch, SearchError>>,
}

impl Search {
    pub(super) fn err(&self) -> Option<&SearchError> {
        self.matches.as_ref()?.as_ref().err()
    }
    pub(super) fn matched_rows(&self) -> &[usize] {
        self.matches
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map_or(&[][..], |m| m.rows())
    }
    /// Whether row `i` matched.
    pub(super) fn contains_row(&self, i: usize) -> bool {
        self.matches
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .is_some_and(|m| m.contains_row(i))
    }
}

/// Search messages, shared by every screen that embeds a [`SearchState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMessage {
    /// Open the modal (`/`) search bar (incremental phase).
    StartModal,
    /// Open an interactive (`C-s`/`C-r`) search in the given direction.
    StartInteractive(SearchDirection),
    /// Append a character to the search pattern.
    Push(char),
    /// Remove the last character from the search pattern.
    Pop,
    /// Fix the current pattern (modal incremental → fixed); empty pattern exits.
    Submit,
    /// Cancel an editing search: restore the origin selection and exit.
    Cancel,
    /// Close the search: keep the current selection.
    Close,
    /// Next match (modal `n` / interactive `C-s`).
    Next,
    /// Previous match (modal `N` / interactive `C-r`).
    Prev,
}

/// What [`SearchState::key_to_message`] made of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKey {
    /// The key is a search message.
    Message(SearchMessage),
    /// The key is eaten by the search without doing anything (an editing
    /// search owns every key).
    Swallow,
    /// Not a search key: the screen maps it as usual.
    Pass,
}

/// Search component embedded in a screen: the active search, if any, and the
/// screen's own history.
#[derive(Debug, Default)]
pub struct SearchState {
    /// The active search, if any.
    pub active: Option<Search>,
    /// Most recently used search pattern, recalled by an empty interactive
    /// search via `C-s`/`C-r`. Shared across modal and interactive searches.
    /// Not `Option` because empty string can represent empty state.
    pub last: String,
    /// How a typed pattern becomes the regex the rows are matched against.
    /// Shared with the session rather than owned, so the migemo process is
    /// spawned once and survives the reloads that rebuild the screens.
    pub translator: Rc<Translator>,
}

impl SearchState {
    /// A state with no active search, remembering `last` as the history and
    /// translating patterns with `translator`.
    pub fn new(last: String, translator: Rc<Translator>) -> Self {
        Self {
            active: None,
            last,
            translator,
        }
    }

    /// Whether an interactive (`C-s`/`C-r`) search is active.
    pub fn is_interactive(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|s| s.intent.mode == SearchMode::Interactive)
    }

    /// Translates a key event into a search message. The editing phases
    /// (modal incremental, interactive i-search) own every key they can use;
    /// the modal fixed phase intercepts only its own controls and lets the
    /// rest fall through so full navigation keeps working. Without an editing
    /// search, `/`, `C-s` and `C-r` start one.
    pub fn key_to_message(&self, key: KeyEvent) -> SearchKey {
        use SearchKey::{Message, Pass, Swallow};
        use SearchMessage::*;
        let ctrl = is_ctrl(key.modifiers);

        if let Some(search) = &self.active {
            match search.intent.mode {
                SearchMode::Modal(SearchPhase::Incremental) => {
                    return match key.code {
                        KeyCode::Esc => Message(Cancel),
                        KeyCode::Enter => Message(Submit),
                        KeyCode::Backspace => Message(Pop),
                        KeyCode::Char(c) if !ctrl => Message(Push(c)),
                        _ => Swallow,
                    };
                }
                SearchMode::Modal(SearchPhase::Fixed) => match key.code {
                    KeyCode::Esc => return Message(Close),
                    KeyCode::Char('n') => return Message(Next),
                    KeyCode::Char('N') => return Message(Prev),
                    _ => {} // fall through to normal UI
                },
                SearchMode::Interactive => match key.code {
                    KeyCode::Char('s') if ctrl => return Message(Next),
                    KeyCode::Char('r') if ctrl => return Message(Prev),
                    KeyCode::Char('g') if ctrl => return Message(Cancel),
                    KeyCode::Esc => return Message(Cancel),
                    KeyCode::Backspace => return Message(Pop),
                    KeyCode::Char(c) if !ctrl => return Message(Push(c)),
                    _ => {} // fall through to normal UI
                },
            }
        }

        match key.code {
            KeyCode::Char('/') => Message(StartModal),
            KeyCode::Char('s') if ctrl => Message(StartInteractive(SearchDirection::Forward)),
            KeyCode::Char('r') if ctrl => Message(StartInteractive(SearchDirection::Backward)),
            _ => Pass,
        }
    }

    /// Applies a [`SearchMessage`], moving `nav` over `rows`.
    pub fn update<T: Searchable>(&mut self, msg: SearchMessage, nav: &mut TableNav, rows: &[T]) {
        match msg {
            SearchMessage::StartModal => self.start(
                SearchMode::Modal(SearchPhase::Incremental),
                SearchDirection::Forward,
                nav,
            ),
            SearchMessage::StartInteractive(dir) => self.start(SearchMode::Interactive, dir, nav),
            SearchMessage::Push(c) => {
                if let Some(search) = self.active.as_mut() {
                    search.intent.input.push(c);
                    search.intent.no_previous = false;
                }
                self.recompute(nav, rows);
            }
            SearchMessage::Pop => {
                if let Some(search) = self.active.as_mut() {
                    search.intent.input.pop();
                    search.intent.no_previous = false;
                }
                self.recompute(nav, rows);
            }
            SearchMessage::Submit => self.submit(),
            SearchMessage::Cancel => {
                if let Some(search) = self.active.take() {
                    // on cancel, search query won't be saved.
                    nav.select_item(search.intent.origin);
                }
            }
            SearchMessage::Close => self.active = None,
            SearchMessage::Next => self.step_or_recall(SearchDirection::Forward, nav, rows),
            SearchMessage::Prev => self.step_or_recall(SearchDirection::Backward, nav, rows),
        }
    }

    /// Opens a search of the given style, recording the current selection as
    /// the origin.
    fn start(&mut self, mode: SearchMode, dir: SearchDirection, nav: &TableNav) {
        let origin = nav.selected_item().unwrap_or(0);
        self.active = Some(Search {
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

    /// Fixes the current pattern (modal incremental → fixed); an empty pattern
    /// exits the search instead.
    fn submit(&mut self) {
        let Some(search) = self.active.as_mut() else {
            return;
        };
        // If empty pattern submitted, simply exits the search mode.
        if search.intent.input.is_empty() {
            self.active = None;
            return;
        }
        self.last = search.intent.input.clone();
        if let SearchMode::Modal(phase) = &mut search.intent.mode {
            *phase = SearchPhase::Fixed;
        }
    }

    /// Ends an active interactive search, keeping the current selection. Used
    /// by keys that both navigate and leave i-search (`C-n`/`C-p`, Enter). A
    /// no-op for modal searches, which stay active during navigation.
    pub fn end_interactive(&mut self) {
        if self.is_interactive()
            // clear search with take().
            && let Some(search) = self.active.take()
            && !search.intent.input.is_empty()
        {
            self.last = search.intent.input;
        }
    }

    /// Handles `C-s`/`C-r` (and modal `n`/`N`). An interactive search on an
    /// empty pattern recalls the last-used pattern (canonical isearch);
    /// otherwise it steps to the next/previous match.
    fn step_or_recall<T: Searchable>(
        &mut self,
        dir: SearchDirection,
        nav: &mut TableNav,
        rows: &[T],
    ) {
        let Some(search) = &mut self.active else {
            return;
        };
        // update direction before operation
        search.intent.dir = dir;
        if search.intent.mode == SearchMode::Interactive && search.intent.input.is_empty() {
            // Restores the history and jumps in `dir`. With no previous
            // pattern, flips on the `[no previous search text]` notice and
            // waits for input.
            search.intent.input = self.last.clone();
            search.intent.no_previous = self.last.is_empty();
            self.recompute(nav, rows);
            return;
        }
        // Moves the selection to the next/previous match (wrapping). No-op
        // without matches.
        let Some(Ok(m)) = search.matches.as_ref() else {
            return;
        };
        let current = nav.selected_item().unwrap_or(0);
        if let Some(next) = m.step(current, dir) {
            nav.select_item(next);
        }
    }

    /// Recompiles the search pattern, recollects matching row indices, and
    /// jumps the selection to the first match in the active direction.
    ///
    /// Modal searches always jump relative to the fixed origin; interactive
    /// searches jump relative to the current point, mirroring isearch. No-op
    /// when no search is active.
    pub fn recompute<T: Searchable>(&mut self, nav: &mut TableNav, rows: &[T]) {
        let Some(search) = self.active.as_mut() else {
            return;
        };
        let intent = &search.intent;
        let origin = intent.origin;
        let reference = match intent.mode {
            SearchMode::Modal(_) => origin,
            SearchMode::Interactive => nav.selected_item().unwrap_or(origin),
        };
        let matches = SearchMatch::compute(&intent.input, rows, &self.translator);
        let jump = match &matches {
            Some(Ok(m)) => m.first_match(reference, intent.dir),
            _ => None,
        };
        search.matches = matches;
        if let Some(idx) = jump {
            nav.select_item(idx);
        }
    }

    /// Owned snapshot of the active search intent, for reload.
    pub fn intent(&self) -> Option<SearchIntent> {
        self.active.as_ref().map(|s| s.intent.clone())
    }

    /// Restores a search captured by [`Self::intent`] (plus the history)
    /// against freshly-built `rows`, recomputing the matches without moving
    /// the selection.
    pub fn restore<T: Searchable>(
        &mut self,
        intent: Option<&SearchIntent>,
        last: &str,
        rows: &[T],
    ) {
        self.last = last.to_owned();
        self.active = intent.map(|intent| {
            let mut intent = intent.clone();
            intent.origin = intent.origin.min(rows.len().saturating_sub(1));
            let matches = SearchMatch::compute(&intent.input, rows, &self.translator);
            Search { intent, matches }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use crossterm::event::KeyModifiers;

    impl Searchable for &str {
        fn is_match(&self, re: &Regex) -> bool {
            re.is_match(self)
        }
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
        let rows: &[&str] = &["Assets", "Income"];
        let plain = Translator::Plain;
        assert_matches!(SearchMatch::compute("", rows, &plain), None);
        assert_matches!(
            SearchMatch::compute("assets", rows, &plain),
            Some(Ok(m)) if m.rows() == [0]
        );
        assert_matches!(
            SearchMatch::compute("[", rows, &plain),
            Some(Err(SearchError::Regex(_)))
        );
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn key_to_message_without_search_only_maps_start_keys() {
        let state = SearchState::default();
        assert_eq!(
            state.key_to_message(key(KeyCode::Char('/'))),
            SearchKey::Message(SearchMessage::StartModal)
        );
        assert_eq!(
            state.key_to_message(ctrl_key('s')),
            SearchKey::Message(SearchMessage::StartInteractive(SearchDirection::Forward))
        );
        assert_eq!(
            state.key_to_message(ctrl_key('r')),
            SearchKey::Message(SearchMessage::StartInteractive(SearchDirection::Backward))
        );
        assert_eq!(
            state.key_to_message(key(KeyCode::Char('n'))),
            SearchKey::Pass
        );
        assert_eq!(state.key_to_message(key(KeyCode::Esc)), SearchKey::Pass);
    }

    #[test]
    fn incremental_search_swallows_unmapped_keys() {
        let mut state = SearchState::default();
        let rows: &[&str] = &[];
        state.update(SearchMessage::StartModal, &mut TableNav::new(0), rows);
        assert_eq!(state.key_to_message(key(KeyCode::Down)), SearchKey::Swallow);
        assert_eq!(
            state.key_to_message(key(KeyCode::Char('j'))),
            SearchKey::Message(SearchMessage::Push('j'))
        );
    }

    /// An invalid pattern can still be submitted; stepping through it then
    /// has no matches to move to, so the selection stays put.
    #[test]
    fn stepping_an_invalid_fixed_pattern_keeps_the_selection() {
        let rows: &[&str] = &["Assets", "Income", "Expenses"];
        let mut nav = TableNav::new(rows.len());
        nav.select_item(1);
        let mut state = SearchState::default();
        state.update(SearchMessage::StartModal, &mut nav, rows);
        state.update(SearchMessage::Push('['), &mut nav, rows);
        state.update(SearchMessage::Submit, &mut nav, rows);
        let search = state.active.as_ref().expect("still searching");
        assert_eq!(search.intent.mode, SearchMode::Modal(SearchPhase::Fixed));
        assert_matches!(search.err(), Some(_));

        state.update(SearchMessage::Next, &mut nav, rows);
        state.update(SearchMessage::Prev, &mut nav, rows);
        assert_eq!(nav.selected_item(), Some(1));
    }
}
