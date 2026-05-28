//! Pure rendering functions for the TUI.

use okane_core::report::ReportContext;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use unicode_width::UnicodeWidthStr;

use super::app::{App, BalanceRow};

const FOOTER_HINT: &str = " ↑/↓ scroll · PgUp/PgDn page · g/G home/end · q quit ";

/// Renders a frame for the given app state.
pub fn draw<'ctx>(frame: &mut Frame, app: &mut App<'ctx>, ctx: &ReportContext<'ctx>) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(1), // title bar
        Constraint::Min(1),    // table body
        Constraint::Length(1), // footer hint
    ])
    .split(area);

    draw_title(frame, layout[0], app);
    draw_body(frame, layout[1], app, ctx);
    draw_footer(frame, layout[2]);
}

fn draw_title(frame: &mut Frame, area: Rect, app: &App<'_>) {
    let title = format!(" okane ui — {} ", app.source_display);
    let paragraph =
        Paragraph::new(Line::from(title)).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(paragraph, area);
}

fn draw_body<'ctx>(frame: &mut Frame, area: Rect, app: &mut App<'ctx>, ctx: &ReportContext<'ctx>) {
    // Account column gets the remaining width; amount column is fixed.
    // The inner area excludes the surrounding border.
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    // Update viewport height (rows visible in the body, minus the header row).
    app.nav.viewport_height = inner.height.saturating_sub(1);

    if app.nav.is_empty() {
        let msg = Paragraph::new("No balances to display")
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![Cell::from("Account"), Cell::from("Amount")])
        .style(Style::default().add_modifier(Modifier::BOLD));

    // Format each row's amount lines once; reused for width and row construction.
    let formatted: Vec<Vec<String>> = app
        .rows
        .iter()
        .map(|row| format_amount_lines(&row.amount, ctx))
        .collect();
    let amount_width = compute_amount_width(&formatted);
    let rows: Vec<Row> = app
        .rows
        .iter()
        .zip(formatted.iter())
        .map(|(row, lines)| make_row(row, lines))
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Min(10), Constraint::Length(amount_width)],
    )
    .header(header)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .block(block);

    frame.render_stateful_widget(table, area, &mut app.nav.table_state);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(FOOTER_HINT).style(Style::default().add_modifier(Modifier::DIM));
    frame.render_widget(footer, area);
}

/// Formats the amount lines for one balance — one line per commodity, or a
/// single `"0"` line when the balance is empty.
fn format_amount_lines<'ctx>(
    amount: &okane_core::report::Amount<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> Vec<String> {
    let mut lines: Vec<String> = amount
        .iter()
        .map(|single| single.as_display(ctx).to_string())
        .collect();
    if lines.is_empty() {
        lines.push("0".to_owned());
    }
    lines
}

/// Builds a multi-line table row for one balance entry. The account label
/// appears only on the first line; remaining lines belong to additional
/// commodities of the same account.
fn make_row<'r>(row: &'r BalanceRow<'_>, lines: &'r [String]) -> Row<'r> {
    let height = row.line_count();
    let account_cell = Cell::from(row.account.as_str());
    let amount_lines: Vec<Line> = lines
        .iter()
        .map(|s| Line::from(s.as_str()).alignment(Alignment::Right))
        .collect();
    let amount_cell = Cell::from(Text::from(amount_lines));
    Row::new(vec![account_cell, amount_cell]).height(height)
}

/// Returns the display width (in columns) needed for the amount column.
fn compute_amount_width(formatted: &[Vec<String>]) -> u16 {
    let max = formatted
        .iter()
        .flat_map(|lines| lines.iter())
        .map(|s| display_width(s))
        .max()
        .unwrap_or(0);
    // Add a 1-column right padding so the value never abuts the border.
    max.saturating_add(1).clamp(10, u16::MAX as usize) as u16
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width_cjk(s)
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use okane_core::report::{Amount, ReportContext};
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn amount_width_has_minimum() {
        let formatted = vec![vec!["1".to_string()]];
        assert_eq!(compute_amount_width(&formatted), 10);
    }

    #[test]
    fn amount_width_grows_with_longest_row() {
        let formatted = vec![
            vec!["1.00 USD".to_string()],
            vec!["12,345,678.90 USD".to_string()],
        ];
        assert_eq!(compute_amount_width(&formatted), 18);
    }

    #[test]
    fn amount_width_considers_all_commodity_lines() {
        let formatted = vec![vec!["1 USD".to_string(), "12,345,678.90 EUR".to_string()]];
        assert_eq!(compute_amount_width(&formatted), 18);
    }

    #[test]
    fn format_amount_lines_empty_falls_back_to_zero() {
        let arena = Bump::new();
        let ctx = ReportContext::new(&arena);
        let amount: Amount<'_> = Amount::zero();
        assert_eq!(format_amount_lines(&amount, &ctx), vec!["0".to_string()]);
    }

    #[test]
    fn format_amount_lines_one_per_commodity() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodity_store_mut().ensure("USD");
        let eur = ctx.commodity_store_mut().ensure("EUR");
        let amount = Amount::from_value(usd, dec!(100)) + Amount::from_value(eur, dec!(200));
        let lines = format_amount_lines(&amount, &ctx);
        assert_eq!(lines.len(), 2);
        // BTreeMap iteration → ordered by CommodityTag (insertion order via interner).
        assert!(lines.iter().any(|s| s == "100 USD"));
        assert!(lines.iter().any(|s| s == "200 EUR"));
    }
}
