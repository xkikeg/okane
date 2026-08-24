use std::cmp::{max, min};
use std::ops::Range;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

use crate::ui::keys::is_ctrl;

/// Rows a page turn keeps from the page it leaves: the new window starts two
/// lines back from where the old one ended (and vice versa going up).
///
/// The overlap is not just for comfort. The top and bottom rows of the body are
/// where a cut account's `+N above` / `+N more` markers go, replacing the
/// amounts on those rows — so a strict page turn would land the line hidden
/// behind one marker straight onto the row holding the other, and its amounts
/// would never be readable on any page. Two rows of overlap carry that line
/// back into the body proper.
const PAGE_OVERLAP: usize = 2;

/// A navigation command over a table — the vocabulary shared by every table
/// screen. Keeps the key bindings and the [`TableNav`] mutations in one place
/// so each screen wraps [`NavCommand`] in its own message rather than
/// re-deriving `move_rows`/`select_first_row` from scratch.
///
/// The report tables draw one row per *line* (an account's balance is one line
/// per commodity), so `Up`/`Down` walk lines while [`NavCommand::NextItem`] and
/// [`NavCommand::PrevItem`] walk whole accounts/entries — see [`LineIndex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavCommand {
    Up,
    Down,
    PageUp,
    PageDown,
    First,
    Last,
    /// First line of the next item (`J`).
    NextItem,
    /// First line of the previous item (`K`).
    PrevItem,
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
        KeyCode::Char('K') => Some(NavCommand::PrevItem),
        KeyCode::Char('J') => Some(NavCommand::NextItem),
        KeyCode::PageUp => Some(NavCommand::PageUp),
        KeyCode::Char('b') if ctrl => Some(NavCommand::PageUp),
        KeyCode::PageDown => Some(NavCommand::PageDown),
        KeyCode::Char('f') if ctrl => Some(NavCommand::PageDown),
        KeyCode::Home | KeyCode::Char('g') => Some(NavCommand::First),
        KeyCode::End | KeyCode::Char('G') => Some(NavCommand::Last),
        _ => None,
    }
}

/// Maps the one-line table rows of a screen to the logical items they belong
/// to: a balance account occupies one row per commodity, a register entry one
/// row per line of its taller amount column.
///
/// Everything the tables draw is a *row* (one terminal line); everything the
/// rest of the UI reasons about — the selected account, a search match, the
/// register a drill-in opens — is an *item*. This is the translation between
/// the two, so neither side has to carry the other's indices.
#[derive(Debug, Default, Clone)]
pub struct LineIndex {
    /// Item -> its first row, ascending. `starts.len()` is the item count, and
    /// everything else is derived from it: a row's item is the last start
    /// at-or-below it. Keeping only the item ends means the index costs one
    /// entry per account rather than one per commodity line.
    starts: Vec<usize>,
    /// Total rows, i.e. where a hypothetical item after the last one would
    /// start.
    row_count: usize,
}

impl LineIndex {
    /// Builds an index from each item's rendered line count. A count of `0` is
    /// treated as `1`: every item occupies at least one row.
    pub fn new(line_counts: impl IntoIterator<Item = u16>) -> Self {
        let mut index = Self::default();
        for count in line_counts {
            index.starts.push(index.row_count);
            index.row_count += usize::from(max(count, 1));
        }
        index
    }

    /// An index over `items` items of one line each — the shape of a table
    /// that needs no exploding (the import screen, the pure nav tests).
    pub fn uniform(items: usize) -> Self {
        Self::new(std::iter::repeat_n(1u16, items))
    }

    /// Number of table rows.
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Number of logical items.
    pub fn item_count(&self) -> usize {
        self.starts.len()
    }

    /// The item a row belongs to.
    pub fn item_at(&self, row: usize) -> Option<usize> {
        if row >= self.row_count {
            return None;
        }
        // Every item starts at or below its own rows, and `starts[0] == 0`, so
        // an in-range row always has one before it.
        Some(self.starts.partition_point(|&start| start <= row) - 1)
    }

    /// The first row of `item`.
    pub fn first_row(&self, item: usize) -> Option<usize> {
        self.starts.get(item).copied()
    }

    /// One past the last row of `item`.
    fn end_row(&self, item: usize) -> usize {
        self.first_row(item + 1).unwrap_or(self.row_count)
    }

    /// The items whose rows intersect `window`, in order, each paired with the
    /// range of *its own* lines that falls inside it.
    ///
    /// This is what a table body is built from: the renderer walks items rather
    /// than rows (it formats an account's amounts once, however many lines they
    /// span) but must draw only the lines the window actually holds, and the
    /// clipped range is also what says whether the item is cut by an edge.
    pub fn items_in(
        &self,
        window: Range<usize>,
    ) -> impl Iterator<Item = (usize, Range<u16>)> + use<'_> {
        let start = min(window.start, self.row_count);
        let end = min(window.end, self.row_count);
        // The first item is the one holding `start`; the last is the one
        // holding `end - 1`, which is one before the first item starting at or
        // after `end`. Both come out of the same search.
        let items = match start < end {
            true => {
                self.starts.partition_point(|&s| s <= start) - 1
                    ..self.starts.partition_point(|&s| s < end)
            }
            false => 0..0,
        };
        items.map(move |item| {
            let first_row = self.starts[item];
            let first_line = (max(start, first_row) - first_row) as u16;
            let end_line = (min(end, self.end_row(item)) - first_row) as u16;
            (item, first_line..end_line)
        })
    }
}

/// Pure scroll/selection state for a table.
///
/// Lives separately from row data so the navigation math can be tested
/// without constructing a `'ctx`-bound `App` or a real `ReportContext`.
#[derive(Debug, Default)]
pub struct TableNav {
    pub table_state: TableState,
    /// Row ↔ item translation for this table; also the source of the row count.
    lines: LineIndex,
    /// Last known viewport height for the table body. Updated each frame and
    /// used to size page-up/page-down jumps.
    pub viewport_height: u16,
    /// Index of the first visible row, maintained by [`Self::visible_window`].
    offset: usize,
}

impl TableNav {
    /// A table of `item_count` single-line items.
    pub fn new(item_count: usize) -> Self {
        Self::from_index(LineIndex::uniform(item_count))
    }

    /// A table whose items occupy the given line counts.
    pub fn with_lines(line_counts: impl IntoIterator<Item = u16>) -> Self {
        Self::from_index(LineIndex::new(line_counts))
    }

    fn from_index(lines: LineIndex) -> Self {
        let mut table_state = TableState::default();
        if lines.row_count() > 0 {
            table_state.select(Some(0));
        }
        Self {
            table_state,
            lines,
            viewport_height: 0,
            offset: 0,
        }
    }

    pub fn lines(&self) -> &LineIndex {
        &self.lines
    }

    pub fn row_count(&self) -> usize {
        self.lines.row_count()
    }

    pub fn item_count(&self) -> usize {
        self.lines.item_count()
    }

    pub fn is_empty(&self) -> bool {
        self.row_count() == 0
    }

    /// Current row selection, defaulting to 0 when nothing is selected.
    ///
    /// A *row* is one line of the table. To address what the row belongs to —
    /// the account or register entry the rest of the UI reasons about — use
    /// [`Self::selected_item`]: the two indices differ as soon as anything
    /// spans more than one commodity, and nothing but the name says which is
    /// which.
    pub fn selected_row(&self) -> usize {
        self.table_state.selected().unwrap_or(0)
    }

    /// The item the selection sits in, if any.
    pub fn selected_item(&self) -> Option<usize> {
        self.lines.item_at(self.table_state.selected()?)
    }

    fn last_row(&self) -> Option<usize> {
        self.row_count().checked_sub(1)
    }

    /// Moves the selection by `delta` rows, clamping to the row range.
    pub fn move_rows(&mut self, delta: isize) {
        let Some(last) = self.last_row() else {
            return;
        };
        let current = self.selected_row() as isize;
        let next = (current + delta).clamp(0, last as isize);
        self.table_state.select(Some(next as usize));
    }

    /// Page size — at least 1 row, falls back to a sensible default if the
    /// viewport height has not been observed yet.
    pub fn page_size(&self) -> usize {
        max(1, self.viewport_height) as usize
    }

    pub fn select_first_row(&mut self) {
        if !self.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn select_last_row(&mut self) {
        if let Some(last) = self.last_row() {
            self.table_state.select(Some(last));
        }
    }

    /// Selects an explicit row index, ignored when out of range.
    pub fn select_row(&mut self, row: usize) {
        if row < self.row_count() {
            self.table_state.select(Some(row));
        }
    }

    /// Selects the first row of `item`, ignored when out of range.
    pub fn select_item(&mut self, item: usize) {
        if let Some(row) = self.lines.first_row(item) {
            self.select_row(row);
        }
    }

    /// Steps the selection a whole item at a time (`J`/`K`), landing on the
    /// neighbour's first row. At either end there is no neighbour to land on,
    /// so the selection goes to that end of the table instead — the key always
    /// moves the way it points.
    fn step_item(&mut self, delta: isize) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let target = item as isize + delta;
        match usize::try_from(target) {
            Ok(target) if target < self.item_count() => self.select_item(target),
            // Past the last item, or before the first.
            _ if delta > 0 => self.select_last_row(),
            _ => self.select_first_row(),
        }
    }

    /// Scrolls the viewport by `pages` page turns and parks the selection on
    /// the edge the movement comes from: the top line when paging down, the
    /// bottom line when paging up.
    ///
    /// Moving the *offset* rather than the selection is what makes each press
    /// turn one page: with the selection alone, a cursor sitting at the top of
    /// the window only has to travel to the bottom for the window to stop
    /// scrolling, so the first press moved one line and only the ones after it
    /// moved a page. A turn is [`PAGE_OVERLAP`] rows short of the full height,
    /// so the reader keeps a foothold on the page they came from.
    ///
    /// Once the offset is against an end, there is no page left to turn, so the
    /// key runs the selection to that end instead — the same "always moves the
    /// way it points" rule [`Self::step_item`] follows.
    fn scroll_page(&mut self, pages: isize) {
        let Some(last) = self.last_row() else {
            return;
        };
        let page = self.page_size();
        // A viewport of one or two rows has no room to overlap and still
        // advance; it moves a row at a time.
        let turn = max(page.saturating_sub(PAGE_OVERLAP), 1);
        let max_offset = self.row_count().saturating_sub(page);
        let offset = (self.offset as isize + pages * turn as isize).clamp(0, max_offset as isize);
        let offset = offset as usize;
        if offset == self.offset {
            match pages > 0 {
                true => self.select_last_row(),
                false => self.select_first_row(),
            }
            return;
        }
        self.offset = offset;
        let row = match pages > 0 {
            true => offset,
            // The bottom line of the window we just scrolled to.
            false => offset + page - 1,
        };
        self.table_state.select(Some(min(row, last)));
    }

    /// Applies a [`NavCommand`].
    pub fn apply(&mut self, cmd: NavCommand) {
        match cmd {
            NavCommand::Up => self.move_rows(-1),
            NavCommand::Down => self.move_rows(1),
            NavCommand::PageUp => self.scroll_page(-1),
            NavCommand::PageDown => self.scroll_page(1),
            NavCommand::First => self.select_first_row(),
            NavCommand::Last => self.select_last_row(),
            NavCommand::NextItem => self.step_item(1),
            NavCommand::PrevItem => self.step_item(-1),
        }
    }

    /// Recomputes the visible window from the current selection, stores the new
    /// offset, and returns the visible `[offset, end)`.
    ///
    /// Thin wrapper over [`visible_window`]: the window math lives there, tested
    /// in isolation; this just feeds it the nav's own fields and writes the
    /// offset back so the caller can't compute it and forget to store it.
    pub fn visible_window(&mut self) -> (usize, usize) {
        let (offset, end) = visible_window(
            self.selected_row(),
            self.offset,
            self.viewport_height,
            self.row_count(),
        );
        self.offset = offset;
        (offset, end)
    }
}

/// Computes the visible row window `[offset, end)`, scrolling the minimum
/// amount needed to keep `selected` in view.
///
/// Every table row is one line tall (tall balances are exploded into one row
/// per commodity by [`LineIndex`]), so the window is just a slice: the renderer
/// builds widget rows only for `[offset, end)` rather than one per entry, which
/// keeps per-frame work proportional to the viewport rather than to the
/// (potentially large) row count.
///
/// Guarantees `offset <= selected < end <= row_count` whenever `row_count > 0`
/// and `selected < row_count`.
///
/// [`TableNav::visible_window`] is the ergonomic entry point; this free function
/// is the pure primitive the unit tests drive directly.
fn visible_window(
    selected: usize,
    prev_offset: usize,
    viewport_height: u16,
    row_count: usize,
) -> (usize, usize) {
    if row_count == 0 {
        return (0, 0);
    }
    // A viewport too short to have a body still shows the selected row; the
    // renderer says so rather than drawing an empty table.
    let budget = max(viewport_height, 1) as usize;
    let selected = min(selected, row_count - 1);
    // The window never starts below the selection; a selection above the
    // previous offset pulls the window up to it (selection at the top).
    let mut offset = min(prev_offset, selected);
    // Never leave the body half empty when there are rows to fill it with.
    offset = min(offset, row_count.saturating_sub(budget));
    // Selection below the window: pin it to the bottom line.
    if selected >= offset + budget {
        offset = selected + 1 - budget;
    }
    (offset, min(offset + budget, row_count))
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

    fn shift(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
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

    /// `J`/`K` are the item-wise siblings of `j`/`k`, and arrive shifted.
    #[test]
    fn key_to_nav_maps_item_steps_to_shifted_j_and_k() {
        assert_eq!(key_to_nav(shift('J')), Some(NavCommand::NextItem));
        assert_eq!(key_to_nav(shift('K')), Some(NavCommand::PrevItem));
        // Unshifted, they stay line-wise.
        assert_eq!(key_to_nav(key(KeyCode::Char('j'))), Some(NavCommand::Down));
        assert_eq!(key_to_nav(key(KeyCode::Char('k'))), Some(NavCommand::Up));
    }

    #[test]
    fn apply_moves_and_jumps() {
        let mut n = nav(5);
        n.viewport_height = 2;
        assert_eq!(n.table_state.selected(), Some(0));
        n.apply(NavCommand::Down);
        assert_eq!(n.table_state.selected(), Some(1));
        // A two-row viewport is all overlap, so a turn is a single row.
        n.apply(NavCommand::PageDown); // window [0,2) -> [1,3), cursor on top
        assert_eq!(n.table_state.selected(), Some(1));
        n.apply(NavCommand::Last);
        assert_eq!(n.table_state.selected(), Some(4));
        n.apply(NavCommand::PageUp); // window [1,3) -> [0,2), cursor at bottom
        assert_eq!(n.table_state.selected(), Some(1));
        n.apply(NavCommand::First);
        assert_eq!(n.table_state.selected(), Some(0));
        n.apply(NavCommand::Up); // clamps at top
        assert_eq!(n.table_state.selected(), Some(0));
    }

    /// The renderer refreshes the offset every frame; a test that pages more
    /// than once has to do the same or the second page starts from a stale
    /// window.
    fn page(n: &mut TableNav, cmd: NavCommand) -> (usize, usize) {
        n.apply(cmd);
        n.visible_window()
    }

    /// Paging down turns the page under a cursor that stays on the top line, so
    /// each press shows the next screenful whole — the cursor no longer has to
    /// walk to the bottom edge before the window moves at all.
    #[test]
    fn page_down_turns_the_page_with_the_cursor_on_top() {
        let mut n = nav(100);
        n.viewport_height = 10;
        assert_eq!(n.visible_window(), (0, 10));

        assert_eq!(page(&mut n, NavCommand::PageDown), (8, 18));
        assert_eq!(n.table_state.selected(), Some(8));
        assert_eq!(page(&mut n, NavCommand::PageDown), (16, 26));
        assert_eq!(n.table_state.selected(), Some(16));
    }

    /// Paging up is the mirror image: the window steps back a page and the
    /// cursor lands on its bottom line.
    #[test]
    fn page_up_turns_the_page_with_the_cursor_at_the_bottom() {
        let mut n = nav(100);
        n.viewport_height = 10;
        n.select_row(55);
        assert_eq!(n.visible_window(), (46, 56));

        assert_eq!(page(&mut n, NavCommand::PageUp), (38, 48));
        assert_eq!(n.table_state.selected(), Some(47));
        assert_eq!(page(&mut n, NavCommand::PageUp), (30, 40));
        assert_eq!(n.table_state.selected(), Some(39));
    }

    /// Every row shows up in the body proper — not only as an edge row, where a
    /// cut account replaces its amounts with a `+N` marker — on some page of a
    /// paging-down run, which is what the overlap buys.
    #[test]
    fn paging_down_leaves_no_row_stuck_on_an_edge() {
        let mut n = nav(100);
        n.viewport_height = 10;
        let mut seen = [false; 100];
        // The very first and last rows of the table have no page before or
        // after them, so they are exempt: nothing can pull them inward.
        seen[0] = true;
        seen[99] = true;
        let mut window = n.visible_window();
        for _ in 0..20 {
            let (start, end) = window;
            // Only the rows strictly inside the body count; the two edge rows
            // are the ones a marker can take over.
            seen[start + 1..end - 1].fill(true);
            window = page(&mut n, NavCommand::PageDown);
        }
        assert_eq!(seen.iter().position(|s| !s), None);
    }

    /// With no page left to turn, the key still moves the way it points.
    #[test]
    fn page_keys_run_to_the_ends_when_the_window_cannot_move() {
        let mut n = nav(25);
        n.viewport_height = 10;

        // Two turns leave the last page showing, cursor on its top line; a
        // third has nowhere to scroll, so it takes the cursor to the end.
        assert_eq!(page(&mut n, NavCommand::PageDown), (8, 18));
        assert_eq!(page(&mut n, NavCommand::PageDown), (15, 25));
        assert_eq!(n.table_state.selected(), Some(15));
        assert_eq!(page(&mut n, NavCommand::PageDown), (15, 25));
        assert_eq!(n.table_state.selected(), Some(24));

        // Same at the top.
        assert_eq!(page(&mut n, NavCommand::PageUp), (7, 17));
        assert_eq!(page(&mut n, NavCommand::PageUp), (0, 10));
        assert_eq!(n.table_state.selected(), Some(9));
        assert_eq!(page(&mut n, NavCommand::PageUp), (0, 10));
        assert_eq!(n.table_state.selected(), Some(0));
    }

    /// A viewport too short to overlap still has to advance, or the key would
    /// do nothing at all.
    #[test]
    fn page_keys_move_a_row_when_the_viewport_is_shorter_than_the_overlap() {
        let mut n = nav(10);
        n.viewport_height = 2;
        assert_eq!(page(&mut n, NavCommand::PageDown), (1, 3));
        assert_eq!(n.table_state.selected(), Some(1));
        assert_eq!(page(&mut n, NavCommand::PageDown), (2, 4));
        assert_eq!(n.table_state.selected(), Some(2));
        assert_eq!(page(&mut n, NavCommand::PageUp), (1, 3));
        assert_eq!(n.table_state.selected(), Some(2));
    }

    /// A table shorter than the viewport has no page to turn at all.
    #[test]
    fn page_keys_on_a_table_that_fits_move_to_the_ends() {
        let mut n = nav(4);
        n.viewport_height = 10;
        assert_eq!(page(&mut n, NavCommand::PageDown), (0, 4));
        assert_eq!(n.table_state.selected(), Some(3));
        assert_eq!(page(&mut n, NavCommand::PageUp), (0, 4));
        assert_eq!(n.table_state.selected(), Some(0));
    }

    #[test]
    fn page_keys_on_empty_table_are_noop() {
        let mut n = nav(0);
        n.viewport_height = 10;
        n.apply(NavCommand::PageDown);
        n.apply(NavCommand::PageUp);
        assert_eq!(n.table_state.selected(), None);
    }

    #[test]
    fn empty_nav_has_no_selection() {
        let n = nav(0);
        assert!(n.is_empty());
        assert_eq!(n.table_state.selected(), None);
    }

    #[test]
    fn move_rows_clamps_to_bounds() {
        let mut n = nav(3);
        assert_eq!(n.table_state.selected(), Some(0));

        n.move_rows(-1);
        assert_eq!(n.table_state.selected(), Some(0));

        n.move_rows(1);
        assert_eq!(n.table_state.selected(), Some(1));

        n.move_rows(100);
        assert_eq!(n.table_state.selected(), Some(2));

        n.move_rows(-100);
        assert_eq!(n.table_state.selected(), Some(0));
    }

    #[test]
    fn select_first_and_last_row() {
        let mut n = nav(5);
        n.select_last_row();
        assert_eq!(n.table_state.selected(), Some(4));
        n.select_first_row();
        assert_eq!(n.table_state.selected(), Some(0));
    }

    #[test]
    fn select_first_or_last_row_on_empty_is_noop() {
        let mut n = nav(0);
        n.select_last_row();
        assert_eq!(n.table_state.selected(), None);
        n.select_first_row();
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

    /// Three items of 1, 3 and 2 lines: rows 0 | 1,2,3 | 4,5.
    fn ragged() -> TableNav {
        TableNav::with_lines([1, 3, 2])
    }

    #[test]
    fn line_index_maps_rows_to_items_and_back() {
        let index = LineIndex::new([1, 3, 2]);
        assert_eq!(index.row_count(), 6);
        assert_eq!(index.item_count(), 3);
        // Every row of item 1 (rows 1..4) reports item 1, its first row and
        // last alike.
        assert_eq!(index.item_at(0), Some(0));
        assert_eq!(index.item_at(1), Some(1));
        assert_eq!(index.item_at(3), Some(1));
        assert_eq!(index.item_at(4), Some(2));
        assert_eq!(index.item_at(6), None);
        assert_eq!(index.first_row(1), Some(1));
        assert_eq!(index.first_row(2), Some(4));
        assert_eq!(index.first_row(3), None);
    }

    /// A zero-line item would make its rows unreachable; it gets one row.
    #[test]
    fn line_index_gives_every_item_at_least_one_row() {
        let index = LineIndex::new([0, 0]);
        assert_eq!(index.row_count(), 2);
        assert_eq!(index.item_at(1), Some(1));
    }

    #[test]
    fn line_index_uniform_is_one_row_per_item() {
        let index = LineIndex::uniform(3);
        assert_eq!(index.row_count(), 3);
        assert_eq!(index.item_count(), 3);
        assert_eq!(index.item_at(2), Some(2));
    }

    /// The renderer walks items but draws rows, so the window has to arrive
    /// already clipped to each item's own lines.
    #[test]
    fn line_index_items_in_clips_each_item_to_the_window() {
        let index = LineIndex::new([1, 3, 2]); // rows 0 | 1,2,3 | 4,5
        let items = |window: Range<usize>| index.items_in(window).collect::<Vec<_>>();

        // The whole table: every item, every line.
        assert_eq!(items(0..6), [(0, 0..1), (1, 0..3), (2, 0..2)]);
        // A window cutting item 1 on both edges yields only its middle line.
        assert_eq!(items(2..3), [(1, 1..2)]);
        // Cut above on the left, cut below on the right.
        assert_eq!(items(3..5), [(1, 2..3), (2, 0..1)]);
        // Out of range is clamped; an empty window yields nothing.
        assert_eq!(items(4..99), [(2, 0..2)]);
        assert_eq!(items(3..3), []);
        assert_eq!(items(9..12), []);
        assert_eq!(LineIndex::default().items_in(0..5).count(), 0);
    }

    #[test]
    fn selected_item_follows_the_row_selection() {
        let mut n = ragged();
        assert_eq!(n.selected_item(), Some(0));
        n.select_row(2);
        assert_eq!(n.selected_item(), Some(1));
        n.select_row(5);
        assert_eq!(n.selected_item(), Some(2));
    }

    #[test]
    fn select_item_lands_on_its_first_row() {
        let mut n = ragged();
        n.select_item(1);
        assert_eq!(n.table_state.selected(), Some(1));
        n.select_item(2);
        assert_eq!(n.table_state.selected(), Some(4));
        // Out of range leaves the selection alone.
        n.select_item(9);
        assert_eq!(n.table_state.selected(), Some(4));
    }

    /// `J`/`K` skip the rest of the current item rather than stepping a line.
    #[test]
    fn item_steps_jump_whole_items() {
        let mut n = ragged();
        n.apply(NavCommand::NextItem);
        assert_eq!(n.table_state.selected(), Some(1)); // item 1, first line
        n.apply(NavCommand::Down);
        assert_eq!(n.table_state.selected(), Some(2)); // still inside item 1
        n.apply(NavCommand::NextItem);
        assert_eq!(n.table_state.selected(), Some(4)); // item 2, first line
        n.apply(NavCommand::PrevItem);
        assert_eq!(n.table_state.selected(), Some(1)); // back to item 1
    }

    /// With no neighbour left, the key still moves the way it points: to the
    /// last row going down, to the first going up.
    #[test]
    fn item_steps_run_to_the_ends_of_the_table() {
        let mut n = ragged();
        n.select_item(2);
        n.apply(NavCommand::NextItem);
        assert_eq!(n.table_state.selected(), Some(5));
        n.apply(NavCommand::NextItem);
        assert_eq!(n.table_state.selected(), Some(5));
        n.select_row(3); // middle of item 1
        n.apply(NavCommand::PrevItem);
        assert_eq!(n.table_state.selected(), Some(0));
        n.apply(NavCommand::PrevItem);
        assert_eq!(n.table_state.selected(), Some(0));
    }

    #[test]
    fn item_steps_on_empty_table_are_noop() {
        let mut n = nav(0);
        n.apply(NavCommand::NextItem);
        n.apply(NavCommand::PrevItem);
        assert_eq!(n.table_state.selected(), None);
    }

    #[test]
    fn empty_table_has_empty_window() {
        assert_eq!(visible_window(0, 0, 10, 0), (0, 0));
    }

    #[test]
    fn everything_fits_shows_all() {
        // 5 rows, viewport of 10: whole table visible, offset stays 0.
        assert_eq!(visible_window(4, 0, 10, 5), (0, 5));
        assert_eq!(visible_window(0, 0, 10, 5), (0, 5));
    }

    #[test]
    fn selection_above_offset_scrolls_up_to_top() {
        // Window was scrolled down to offset 20; selecting row 5 pulls it up so
        // row 5 sits at the top.
        assert_eq!(visible_window(5, 20, 10, 100), (5, 15));
    }

    #[test]
    fn selection_within_window_keeps_offset() {
        // offset 10, viewport 10 → rows [10, 20); selecting 15 changes nothing.
        assert_eq!(visible_window(15, 10, 10, 100), (10, 20));
    }

    #[test]
    fn selection_below_window_pins_to_bottom() {
        // offset 10, viewport 10 → rows [10, 20); selecting 25 scrolls down so
        // 25 is the last visible row.
        assert_eq!(visible_window(25, 10, 10, 100), (16, 26));
    }

    #[test]
    fn jump_to_last_row_shows_final_page() {
        // g→G style jump from the top: last row pinned to the bottom.
        assert_eq!(visible_window(99, 0, 10, 100), (90, 100));
    }

    #[test]
    fn jump_to_first_row_shows_first_page() {
        assert_eq!(visible_window(0, 90, 10, 100), (0, 10));
    }

    /// A stale offset from a longer table must not leave the body half empty.
    #[test]
    fn offset_is_pulled_back_when_the_table_shrinks() {
        assert_eq!(visible_window(9, 8, 10, 10), (0, 10));
    }

    #[test]
    fn zero_viewport_shows_only_the_selection() {
        assert_eq!(visible_window(3, 0, 0, 10), (3, 4));
    }
}
