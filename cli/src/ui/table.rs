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
