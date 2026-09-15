//! Account-name search state for the balance screen.
//!
//! Pure data and matching logic: [`SearchIntent`] captures *what* the user is
//! searching for and how, [`SearchMatch`] holds the computed row indices, and
//! [`Search`] pairs the two. The orchestration that mutates these against a
//! live balance view lives on [`super::balance::BalanceView`].

use std::borrow::Cow;

use regex::RegexBuilder;

use crate::ui::migemo::{Migemo, MigemoError};

use super::balance::BalanceRow;

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

    /// Computes matching row indices for `input`, translated by `translator`
    /// and compiled as a case-insensitive regex. Returns `None` for empty
    /// input, `Err` for a pattern that cannot be built.
    pub fn compute(
        input: &str,
        rows: &[BalanceRow<'_>],
        translator: &Translator,
    ) -> Option<Result<Self, SearchError>> {
        if input.is_empty() {
            return None;
        }
        Some(Self::compute_nonempty(input, rows, translator))
    }

    fn compute_nonempty(
        input: &str,
        rows: &[BalanceRow<'_>],
        translator: &Translator,
    ) -> Result<Self, SearchError> {
        let pattern = translator.to_regex(input)?;
        let re = RegexBuilder::new(&pattern).case_insensitive(true).build()?;
        Ok(Self(
            rows.iter()
                .enumerate()
                .filter(|(_, row)| re.is_match(row.full_name()))
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

/// Account-name search state on the balance screen.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;

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
        let plain = Translator::Plain;
        assert_matches!(SearchMatch::compute("", rows, &plain), None);
        assert_matches!(SearchMatch::compute("assets", rows, &plain), Some(Ok(_)));
        assert_matches!(
            SearchMatch::compute("[", rows, &plain),
            Some(Err(SearchError::Regex(_)))
        );
    }
}
