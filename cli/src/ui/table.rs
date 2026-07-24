use std::cmp::max;

use ratatui::widgets::TableState;

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
pub fn visible_window(
    selected: usize,
    prev_offset: usize,
    viewport_height: u16,
    row_count: usize,
    height_of: impl Fn(usize) -> u16,
) -> (usize, usize) {
    if row_count == 0 {
        return (0, 0);
    }
    let budget = u32::from(viewport_height);
    // The window never starts below the selection; a selection above the
    // previous offset pulls the window up to it (selection at the top).
    let offset = prev_offset.min(selected);
    let end = fill_from(offset, budget, row_count, &height_of);
    if selected < end {
        return (offset, end);
    }
    // Selection sits below the window: pin it to the bottom by walking up from
    // it, accumulating heights until the viewport is full.
    let mut offset = selected;
    let mut used = u32::from(height_of(selected));
    while offset > 0 {
        let h = u32::from(height_of(offset - 1));
        if used + h > budget {
            break;
        }
        used += h;
        offset -= 1;
    }
    (offset, selected + 1)
}

/// Largest `end` such that rows `[start, end)` fit within `budget` lines,
/// always including at least the `start` row (even if it alone overflows).
fn fill_from(
    start: usize,
    budget: u32,
    row_count: usize,
    height_of: &impl Fn(usize) -> u16,
) -> usize {
    let mut end = start;
    let mut used = 0u32;
    while end < row_count {
        let h = u32::from(height_of(end));
        if end > start && used + h > budget {
            break;
        }
        used += h;
        end += 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // row0(1)+row1(2)+row2(1) = 4, +row3(2)=6 > 5 → stop → rows [0, 3).
        let heights = |i: usize| if i.is_multiple_of(2) { 1 } else { 2 };
        assert_eq!(visible_window(0, 0, 5, 10, heights), (0, 3));
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
