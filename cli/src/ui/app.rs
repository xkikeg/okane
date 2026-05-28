//! UI application state.

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

/// Number of lines an [`Amount`] would render as in the balance table.
///
/// Exposed at module level so it can be tested without constructing an
/// `Account<'ctx>`, which has no public constructor outside `okane_core`.
fn amount_line_count(amount: &Amount<'_>) -> u16 {
    let n = amount.iter().count();
    n.max(1).min(u16::MAX as usize) as u16
}

/// Pure scroll/selection state for the balance table.
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
    pub should_quit: bool,
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
            should_quit: false,
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

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

/// Application state for the TUI session.
#[derive(Debug)]
pub struct App<'ctx> {
    pub source_display: String,
    pub rows: Vec<BalanceRow<'ctx>>,
    pub nav: TableNav,
}

impl<'ctx> App<'ctx> {
    pub fn new(source_display: String, rows: Vec<BalanceRow<'ctx>>) -> Self {
        let nav = TableNav::new(rows.len());
        Self {
            source_display,
            rows,
            nav,
        }
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
}
