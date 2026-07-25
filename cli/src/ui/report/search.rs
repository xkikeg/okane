//! Account-name search state for the balance screen.
//!
//! Pure data and matching logic: [`SearchIntent`] captures *what* the user is
//! searching for and how, [`SearchMatch`] holds the computed row indices, and
//! [`Search`] pairs the two. The orchestration that mutates these against a
//! live balance view lives on [`super::balance::BalanceView`].

use regex::RegexBuilder;

use super::balance::BalanceRow;

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
                            .filter(|(_, row)| re.is_match(row.full_name()))
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
        assert_matches!(SearchMatch::compute("", rows), None);
        assert_matches!(SearchMatch::compute("assets", rows), Some(Ok(_)));
        assert_matches!(SearchMatch::compute("[", rows), Some(Err(_)));
    }
}
