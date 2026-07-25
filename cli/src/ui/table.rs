use std::cmp::{max, min};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

use crate::ui::keys::is_ctrl;

/// A navigation command over a table — the vocabulary shared by every table
/// screen. Keeps the key bindings and the [`TableNav`] mutations in one place
/// so each screen wraps [`NavCommand`] in its own message rather than
/// re-deriving `move_selection`/`select_first` from scratch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavCommand {
    Up,
    Down,
    PageUp,
    PageDown,
    First,
    Last,
}

/// Maps a key event to a [`NavCommand`], the navigation bindings shared by the
/// balance and register tables (and mirrored by the error-popup scroll).
/// Returns `None` for keys that aren't navigation.
pub fn key_to_nav(key: KeyEvent) -> Option<NavCommand> {
    let ctrl = is_ctrl(key.modifiers);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(NavCommand::Up),
        KeyCode::Char('p') if ctrl => Some(NavCommand::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(NavCommand::Down),
        KeyCode::Char('n') if ctrl => Some(NavCommand::Down),
        KeyCode::PageUp => Some(NavCommand::PageUp),
        KeyCode::Char('b') if ctrl => Some(NavCommand::PageUp),
        KeyCode::PageDown => Some(NavCommand::PageDown),
        KeyCode::Char('f') if ctrl => Some(NavCommand::PageDown),
        KeyCode::Home | KeyCode::Char('g') => Some(NavCommand::First),
        KeyCode::End | KeyCode::Char('G') => Some(NavCommand::Last),
        _ => None,
    }
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
    /// Index of the first visible row, maintained by the register renderer's
    /// virtualization ([`visible_window`]). The balance table leaves this at 0
    /// and relies on ratatui's built-in scrolling instead.
    pub offset: usize,
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
            offset: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    /// Current selection index, defaulting to 0 when nothing is selected.
    pub fn selected_index(&self) -> usize {
        self.table_state.selected().unwrap_or(0)
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
        max(1, self.viewport_height) as usize
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

    /// Applies a [`NavCommand`].
    pub fn apply(&mut self, cmd: NavCommand) {
        match cmd {
            NavCommand::Up => self.move_selection(-1),
            NavCommand::Down => self.move_selection(1),
            NavCommand::PageUp => {
                let delta = -(self.page_size() as isize);
                self.move_selection(delta);
            }
            NavCommand::PageDown => {
                let delta = self.page_size() as isize;
                self.move_selection(delta);
            }
            NavCommand::First => self.select_first(),
            NavCommand::Last => self.select_last(),
        }
    }

    /// Recomputes the virtualized window from the current selection, stores the
    /// new [`offset`](Self::offset), and returns the visible `[offset, end)`.
    /// `height_of(i)` gives row `i`'s rendered line count (always `>= 1`).
    ///
    /// Thin wrapper over [`visible_window`]: the window math lives there, tested
    /// in isolation; this just feeds it the nav's own fields and writes the
    /// offset back so the caller can't compute it and forget to store it.
    pub fn visible_window(&mut self, height_of: impl Fn(usize) -> u16) -> (usize, usize) {
        let selected = self.selected_index();
        let (offset, end) = visible_window(
            selected,
            self.offset,
            self.viewport_height,
            self.row_count,
            height_of,
        );
        self.offset = offset;
        (offset, end)
    }
}

/// Computes the visible row window `[offset, end)` for a variable-height table,
/// scrolling the minimum amount needed to keep `selected` in view.
///
/// This is the virtualization primitive for the register table: rather than
/// building a widget row for every entry (O(n) per frame), the renderer builds
/// rows only for `[offset, end)`. `height_of(i)` returns the rendered line
/// count of row `i` (always `>= 1`); `viewport_height` is the number of body
/// lines available.
///
/// Guarantees `offset <= selected < end <= row_count` whenever `row_count > 0`
/// and `selected < row_count`. Both scans touch at most a viewport's worth of
/// rows, so it stays O(viewport) even when `selected` jumps to either end.
///
/// [`TableNav::visible_window`] is the ergonomic entry point; this free function
/// is the pure primitive the unit tests drive directly.
fn visible_window(
    selected: usize,
    prev_offset: usize,
    viewport_height: u16,
    row_count: usize,
    height_of: impl Fn(usize) -> u16,
) -> (usize, usize) {
    if row_count == 0 {
        return (0, 0);
    }
    let budget = viewport_height;
    // The window never starts below the selection; a selection above the
    // previous offset pulls the window up to it (selection at the top).
    let offset = min(prev_offset, selected);
    let end = fill_forward(offset, budget, row_count, &height_of);
    if selected < end {
        return (offset, end);
    }
    // Selection sits below the window: pin it to the bottom by walking up from
    // it, accumulating heights until the viewport is full.
    (fill_backward(selected, budget, &height_of), selected + 1)
}

/// Smallest `end` such that rows `start..end` fulfill the `budget` lines.
fn fill_forward(
    start: usize,
    budget: u16,
    row_count: usize,
    height_of: &impl Fn(usize) -> u16,
) -> usize {
    let mut end = start;
    let mut used = 0u16;
    while end < row_count && used < budget {
        used += height_of(end);
        end += 1;
    }
    end
}

/// Largest `start` such that rows `start..=last` fulfill the `budget` lines.
fn fill_backward(last: usize, budget: u16, height_of: &impl Fn(usize) -> u16) -> usize {
    let mut start = last;
    let mut used = 0u16;
    loop {
        used += height_of(start);
        if start == 0 || used >= budget {
            break;
        }
        start -= 1;
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;

    use crossterm::event::KeyModifiers;
    use pretty_assertions::assert_eq;

    fn nav(n: usize) -> TableNav {
        TableNav::new(n)
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn key_to_nav_maps_the_shared_bindings() {
        assert_eq!(key_to_nav(key(KeyCode::Down)), Some(NavCommand::Down));
        assert_eq!(key_to_nav(key(KeyCode::Char('j'))), Some(NavCommand::Down));
        assert_eq!(key_to_nav(ctrl('n')), Some(NavCommand::Down));
        assert_eq!(key_to_nav(key(KeyCode::Up)), Some(NavCommand::Up));
        assert_eq!(key_to_nav(ctrl('p')), Some(NavCommand::Up));
        assert_eq!(
            key_to_nav(key(KeyCode::PageDown)),
            Some(NavCommand::PageDown)
        );
        assert_eq!(key_to_nav(ctrl('f')), Some(NavCommand::PageDown));
        assert_eq!(key_to_nav(ctrl('b')), Some(NavCommand::PageUp));
        assert_eq!(key_to_nav(key(KeyCode::Home)), Some(NavCommand::First));
        assert_eq!(key_to_nav(key(KeyCode::Char('g'))), Some(NavCommand::First));
        assert_eq!(key_to_nav(key(KeyCode::Char('G'))), Some(NavCommand::Last));
        // Non-navigation keys are left for the caller.
        assert_eq!(key_to_nav(key(KeyCode::Char('q'))), None);
        assert_eq!(key_to_nav(key(KeyCode::Enter)), None);
        assert_eq!(key_to_nav(key(KeyCode::Char('f'))), None); // f without ctrl
    }

    #[test]
    fn apply_moves_and_jumps() {
        let mut n = nav(5);
        n.viewport_height = 2;
        assert_eq!(n.table_state.selected(), Some(0));
        n.apply(NavCommand::Down);
        assert_eq!(n.table_state.selected(), Some(1));
        n.apply(NavCommand::PageDown); // +2
        assert_eq!(n.table_state.selected(), Some(3));
        n.apply(NavCommand::Last);
        assert_eq!(n.table_state.selected(), Some(4));
        n.apply(NavCommand::PageUp); // -2
        assert_eq!(n.table_state.selected(), Some(2));
        n.apply(NavCommand::First);
        assert_eq!(n.table_state.selected(), Some(0));
        n.apply(NavCommand::Up); // clamps at top
        assert_eq!(n.table_state.selected(), Some(0));
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

    /// All rows one line tall — the common register case.
    fn uniform(_i: usize) -> u16 {
        1
    }

    #[test]
    fn empty_table_has_empty_window() {
        assert_eq!(visible_window(0, 0, 10, 0, uniform), (0, 0));
    }

    #[test]
    fn everything_fits_shows_all() {
        // 5 rows, viewport of 10: whole table visible, offset stays 0.
        assert_eq!(visible_window(4, 0, 10, 5, uniform), (0, 5));
        assert_eq!(visible_window(0, 0, 10, 5, uniform), (0, 5));
    }

    #[test]
    fn selection_above_offset_scrolls_up_to_top() {
        // Window was scrolled down to offset 20; selecting row 5 pulls it up so
        // row 5 sits at the top.
        assert_eq!(visible_window(5, 20, 10, 100, uniform), (5, 15));
    }

    #[test]
    fn selection_within_window_keeps_offset() {
        // offset 10, viewport 10 → rows [10, 20); selecting 15 changes nothing.
        assert_eq!(visible_window(15, 10, 10, 100, uniform), (10, 20));
    }

    #[test]
    fn selection_below_window_pins_to_bottom() {
        // offset 10, viewport 10 → rows [10, 20); selecting 25 scrolls down so
        // 25 is the last visible row.
        assert_eq!(visible_window(25, 10, 10, 100, uniform), (16, 26));
    }

    #[test]
    fn jump_to_last_row_shows_final_page() {
        // g→G style jump from the top: last row pinned to the bottom.
        assert_eq!(visible_window(99, 0, 10, 100, uniform), (90, 100));
    }

    #[test]
    fn jump_to_first_row_shows_first_page() {
        assert_eq!(visible_window(0, 90, 10, 100, uniform), (0, 10));
    }

    #[test]
    fn variable_heights_respect_line_budget() {
        // Rows alternate 1 and 2 lines tall. Viewport of 5 lines from offset 0:
        // row0(1)+row1(2)+row2(1) = 4, +row3(2)=6 > 5 → stop → rows [0, 4).
        let heights = |i: usize| if i.is_multiple_of(2) { 1 } else { 2 };
        assert_eq!(visible_window(0, 0, 5, 10, heights), (0, 4));
    }

    #[test]
    fn variable_heights_pin_bottom_accounts_for_tall_rows() {
        // Same alternating heights, viewport 5. Select row 5 (2 lines tall):
        // walk up 5(2),4(1),3(2)=5 fits, 2(1)=6>5 stop → offset 3, end 6.
        let heights = |i: usize| if i.is_multiple_of(2) { 1 } else { 2 };
        assert_eq!(visible_window(5, 0, 5, 10, heights), (3, 6));
    }

    #[test]
    fn row_taller_than_viewport_still_shows_selection() {
        // A single 4-line row in a 2-line viewport: show it alone.
        let heights = |_i: usize| 4u16;
        assert_eq!(visible_window(7, 0, 2, 10, heights), (7, 8));
    }

    #[test]
    fn zero_viewport_shows_only_selection() {
        assert_eq!(visible_window(3, 0, 0, 10, uniform), (3, 4));
    }
}
