//! Pure rendering functions for the TUI.
//!
//! The popup pattern follows
//! <https://ratatui.rs/recipes/layout/center-a-widget/>:
//! render a `Clear` widget over a centered rect, then render the popup
//! contents on top.

use std::cmp::{max, min};

use okane_core::report::{Amount, ReportContext};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use unicode_width::UnicodeWidthStr;

use super::app::{
    App, BalanceRow, ErrorPopup, Overlay, RegisterRow, RegisterView, Screen, Search,
    SearchDirection, SearchMatch, SearchMode, SearchPhase,
};
use crate::ui::table::TableNav;

const FOOTER_HINT_BALANCE: &str = " ↑/↓ scroll · PgUp/PgDn page · g/G home/end · Enter register · / search · C-s isearch · r reload · q quit ";
const FOOTER_HINT_REGISTER: &str =
    " ↑/↓ scroll · PgUp/PgDn page · g/G home/end · r reload · q/Esc back ";
const ERROR_POPUP_HINT: &str =
    " ↑/↓ scroll · PgUp/PgDn page · g/G top/end · r reload · Esc/Enter close · q quit ";

/// Renders a frame for the given app state.
pub fn draw<'ctx>(frame: &mut Frame, app: &mut App<'ctx>, ctx: &ReportContext<'ctx>) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(1), // title bar
        Constraint::Min(1),    // body
        Constraint::Length(1), // footer hint
    ])
    .split(area);

    draw_title(frame, layout[0], app);
    match &mut app.screen {
        Screen::Balance => {
            let matches = app
                .search
                .as_ref()
                .and_then(|s| s.matches.as_ref())
                .and_then(|r| r.as_ref().ok());
            draw_balance_body(
                frame,
                layout[1],
                &app.balance_rows,
                &mut app.balance_nav,
                matches,
                ctx,
            );
            match (&app.error_toast, &app.search) {
                (Some(msg), _) => draw_error(frame, layout[2], msg),
                (None, Some(search)) => draw_search_bar(frame, layout[2], search),
                (None, None) => draw_footer(frame, layout[2], FOOTER_HINT_BALANCE),
            }
        }
        Screen::Register(view) => {
            draw_register_body(frame, layout[1], view, ctx);
            match &app.error_toast {
                Some(msg) => draw_error(frame, layout[2], msg),
                None => draw_footer(frame, layout[2], FOOTER_HINT_REGISTER),
            }
        }
    }

    match &mut app.overlay {
        Some(Overlay::QuitConfirm) => draw_quit_confirm(frame, area),
        Some(Overlay::Error(popup)) => draw_error_popup(frame, area, popup),
        None => {}
    }
}

fn draw_title(frame: &mut Frame, area: Rect, app: &App<'_>) {
    let title = match &app.screen {
        Screen::Balance => format!(" okane ui — {} ", app.source_display),
        Screen::Register(view) => format!(
            " okane ui — {} — register: {} ",
            app.source_display,
            view.account.as_str()
        ),
    };
    let paragraph =
        Paragraph::new(Line::from(title)).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(paragraph, area);
}

fn draw_balance_body<'ctx>(
    frame: &mut Frame,
    area: Rect,
    rows: &[BalanceRow<'ctx>],
    nav: &mut TableNav,
    matches: Option<&SearchMatch>,
    ctx: &ReportContext<'ctx>,
) {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    nav.viewport_height = inner.height.saturating_sub(1);

    if rows.is_empty() {
        let msg = Paragraph::new("No balances to display")
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![Cell::from("Account"), Cell::from("Amount")])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let formatted: Vec<Vec<String>> = rows
        .iter()
        .map(|row| format_amount_lines(&row.amount, ctx))
        .collect();
    let amount_width = compute_amount_width(&formatted);
    let table_rows: Vec<Row> = rows
        .iter()
        .zip(formatted.iter())
        .enumerate()
        .map(|(i, (row, lines))| {
            make_balance_row(row, lines, matches.is_some_and(|m| m.contains_row(i)))
        })
        .collect();

    let table = Table::new(
        table_rows,
        [Constraint::Min(10), Constraint::Length(amount_width)],
    )
    .header(header)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .block(block);

    frame.render_stateful_widget(table, area, &mut nav.table_state);
}

fn draw_register_body<'ctx>(
    frame: &mut Frame,
    area: Rect,
    view: &mut RegisterView<'ctx>,
    ctx: &ReportContext<'ctx>,
) {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    view.nav.viewport_height = inner.height.saturating_sub(1);

    if view.rows.is_empty() {
        let msg = Paragraph::new("No register entries to display")
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Date"),
        Cell::from("Payee"),
        Cell::from("Amount"),
        Cell::from("Total"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let amount_lines: Vec<Vec<String>> = view
        .rows
        .iter()
        .map(|row| format_amount_lines(&row.amount, ctx))
        .collect();
    let total_lines: Vec<Vec<String>> = view
        .rows
        .iter()
        .map(|row| format_amount_lines(&row.total, ctx))
        .collect();
    let amount_width = compute_amount_width(&amount_lines);
    let total_width = compute_amount_width(&total_lines);

    let table_rows: Vec<Row> = view
        .rows
        .iter()
        .zip(amount_lines.iter())
        .zip(total_lines.iter())
        .map(|((row, amt), tot)| make_register_row(row, amt, tot))
        .collect();

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(10), // YYYY-MM-DD
            Constraint::Min(10),    // payee, takes remaining
            Constraint::Length(amount_width),
            Constraint::Length(total_width),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .block(block);

    frame.render_stateful_widget(table, area, &mut view.nav.table_state);
}

fn draw_footer(frame: &mut Frame, area: Rect, hint: &str) {
    let footer = Paragraph::new(hint).style(Style::default().add_modifier(Modifier::DIM));
    frame.render_widget(footer, area);
}

/// Renders a transient error notice (e.g. a failed reload) in the footer
/// slot. Takes priority over the key hint and the search bar; cleared by
/// [`App::update`] on the next key press.
fn draw_error(frame: &mut Frame, area: Rect, message: &str) {
    let footer = Paragraph::new(format!(" {message} ")).style(Style::default().fg(Color::Red));
    frame.render_widget(footer, area);
}

/// Renders the balance search bar in the footer slot: a prompt + the typed
/// pattern (red on an invalid regex) plus a dim hint suffix. While editing it
/// also places the terminal cursor at the end of the pattern.
fn draw_search_bar(frame: &mut Frame, area: Rect, search: &Search) {
    let count = search.matched_rows().len();
    let (prompt, hint_and_match, show_cursor): (&str, String, bool) = match search.intent.mode {
        SearchMode::Modal(SearchPhase::Incremental) => ("/", format!("  [{count} matches]"), true),
        SearchMode::Modal(SearchPhase::Fixed) => (
            "/",
            format!("  [{count} matches · n/N next/prev · Esc exit]"),
            false,
        ),
        SearchMode::Interactive => {
            let prompt = match search.intent.dir {
                SearchDirection::Forward => "I-search: ",
                SearchDirection::Backward => "I-search backward: ",
            };
            (
                prompt,
                format!("  [{count} matches · C-s/C-r next/prev · RET exit · C-g cancel]"),
                true,
            )
        }
    };
    let alert = match (search.err(), search.intent.no_previous) {
        (Some(_), _) => Span::styled(
            "  [invalid regex]".to_string(),
            Style::default().fg(Color::Red),
        ),
        (None, true) => Span::styled(
            "  [no previous search text]".to_owned(),
            Style::default().fg(Color::Yellow),
        ),
        (None, false) => Span::default(),
    };
    let text = format!("{prompt}{}", search.intent.input);
    let text_style = if search.err().is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };
    let line = Line::from(vec![
        Span::styled(&text, text_style),
        alert,
        Span::styled(hint_and_match, Style::default().add_modifier(Modifier::DIM)),
    ]);
    frame.render_widget(Paragraph::new(line), area);

    if show_cursor {
        // Cursor sits just past the last typed character (after the prompt),
        // clamped so it never leaves the footer row.
        let cursor_x = area.x.saturating_add(display_width(&text) as u16);
        let cursor_x = min(cursor_x, area.x + area.width.saturating_sub(1));
        frame.set_cursor_position((cursor_x, area.y));
    }
}

/// Centered modal asking the user to confirm quitting.
fn draw_quit_confirm(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(area, 40, 5);
    // Clear first so the popup paints over the table cleanly.
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Quit? ")
        .style(Style::default().add_modifier(Modifier::BOLD));
    let body = Paragraph::new(vec![
        Line::from(""),
        Line::from("Quit okane ui? (y/n)").alignment(Alignment::Center),
    ])
    .block(block);
    frame.render_widget(body, popup);
}

/// Near-full-screen modal showing a failure in full, e.g. a reload that could
/// not parse the ledger.
///
/// The body is deliberately *not* wrapped: annotate-snippets output aligns a
/// line-number gutter, the source line and a caret row by column, and soft
/// wrapping would move the carets away from what they point at. Long lines are
/// clipped instead. That also keeps `lines.len()` the rendered line count, so
/// [`ErrorPopup::scroll`] can clamp exactly.
fn draw_error_popup(frame: &mut Frame, area: Rect, popup: &mut ErrorPopup) {
    let rect = error_popup_rect(area);
    // Clear first so the popup paints over the table cleanly.
    frame.render_widget(Clear, rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(popup.title.as_str())
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);
    // The body height is only known here; keep it fresh for the scroll math,
    // the same way the tables refresh `TableNav::viewport_height`.
    popup.viewport_height = layout[0].height;
    popup.clamp();

    let body: Vec<Line> = popup
        .lines
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect();
    frame.render_widget(
        Paragraph::new(body)
            .style(
                Style::default()
                    .fg(Color::Reset)
                    .remove_modifier(Modifier::BOLD),
            )
            .scroll((popup.scroll, 0)),
        layout[0],
    );

    // Split so the position counter survives on narrow terminals; the hint
    // is the part that gets clipped.
    let position = format!(" {}/{} ", popup.scroll + 1, popup.lines.len());
    let dim = Style::default().add_modifier(Modifier::DIM);
    let footer = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(display_width(&position) as u16),
    ])
    .split(layout[1]);
    frame.render_widget(Paragraph::new(ERROR_POPUP_HINT).style(dim), footer[0]);
    frame.render_widget(Paragraph::new(position).style(dim), footer[1]);
}

/// Four fifths of the screen, with a floor so tiny terminals still show
/// something usable.
fn error_popup_rect(area: Rect) -> Rect {
    let width = (u32::from(area.width) * 4 / 5) as u16;
    let height = (u32::from(area.height) * 4 / 5) as u16;
    centered_rect(area, max(width, 20), max(height, 5))
}

/// Returns a sub-rect of `area` of the given size, centered.
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = min(width, area.width);
    let height = min(height, area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// Formats the amount lines for one balance/register cell — one line per
/// commodity, or a single `"0"` line when the balance is empty.
fn format_amount_lines<'ctx>(amount: &Amount<'ctx>, ctx: &ReportContext<'ctx>) -> Vec<String> {
    let mut lines: Vec<String> = amount
        .iter()
        .map(|single| single.as_display(ctx).to_string())
        .collect();
    if lines.is_empty() {
        lines.push("0".to_owned());
    }
    lines
}

fn make_balance_row<'r>(row: &'r BalanceRow<'_>, lines: &'r [String], is_match: bool) -> Row<'r> {
    let height = row.line_count();
    let mut account_cell = Cell::from(row.account.as_str());
    if is_match {
        account_cell = account_cell.style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    }
    let amount_cell = Cell::from(amount_text(lines));
    Row::new(vec![account_cell, amount_cell]).height(height)
}

fn make_register_row<'r>(
    row: &'r RegisterRow<'_>,
    amount_lines: &'r [String],
    total_lines: &'r [String],
) -> Row<'r> {
    let height = row.line_count();
    let date_cell = Cell::from(row.date.to_string());
    let payee_cell = Cell::from(row.payee.as_str());
    let amount_cell = Cell::from(amount_text(amount_lines));
    let total_cell = Cell::from(amount_text(total_lines));
    Row::new(vec![date_cell, payee_cell, amount_cell, total_cell]).height(height)
}

fn amount_text<'a>(lines: &'a [String]) -> Text<'a> {
    let lines: Vec<Line<'a>> = lines
        .iter()
        .map(|s| Line::from(s.as_str()).alignment(Alignment::Right))
        .collect();
    Text::from(lines)
}

/// Returns the display width (in columns) needed for an amount column.
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
    use std::path::Path;

    use bumpalo::Bump;
    use okane_core::report::query::DateRange;
    use okane_core::report::{Amount, ReportContext};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Position;
    use rust_decimal_macros::dec;

    use super::super::app::{Message, RegisterQueryTemplate, ScrollDelta};
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
        assert!(lines.iter().any(|s| s == "100 USD"));
        assert!(lines.iter().any(|s| s == "200 EUR"));
    }

    #[test]
    fn centered_rect_sits_in_the_middle() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 30,
        };
        let r = centered_rect(area, 40, 6);
        assert_eq!(r.width, 40);
        assert_eq!(r.height, 6);
        assert_eq!(r.x, 30);
        assert_eq!(r.y, 12);
    }

    #[test]
    fn centered_rect_clamps_to_area() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 4,
        };
        let r = centered_rect(area, 40, 6);
        assert_eq!(r.width, 10);
        assert_eq!(r.height, 4);
    }

    #[test]
    fn error_popup_rect_is_four_fifths() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 30,
        };
        let r = error_popup_rect(area);
        assert_eq!(r.width, 80);
        assert_eq!(r.height, 24);
    }

    #[test]
    fn error_popup_rect_has_a_floor_on_tiny_terminals() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 12,
            height: 4,
        };
        // The floor exceeds the area, so it clamps back to the whole screen.
        let r = error_popup_rect(area);
        assert_eq!(r.width, 12);
        assert_eq!(r.height, 4);
    }

    /// Renders `app` into an 80×24 headless terminal and returns the plain text.
    fn render<'ctx>(app: &mut App<'ctx>, ctx: &ReportContext<'ctx>) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app, ctx)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[Position { x, y }].symbol());
            }
            s.push('\n');
        }
        s
    }

    fn golden(name: &str) -> okane_golden::Golden {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../testdata/report/ui")
            .join(format!("{name}.txt"));
        okane_golden::Golden::new(path).unwrap()
    }

    /// The state a failed reload leaves behind: no rows, error modal up.
    fn app_with_error_modal<'ctx>() -> App<'ctx> {
        let mut app = App::new(
            "test.ledger".to_owned(),
            Vec::new(),
            RegisterQueryTemplate {
                conversion: None,
                date_range: DateRange::default(),
            },
        );
        let lines = (0..40).map(|i| format!("error line {i}")).collect();
        app.overlay = Some(Overlay::Error(ErrorPopup::new(
            " failed to load test.ledger ".to_owned(),
            lines,
        )));
        app
    }

    #[test]
    fn render_error_modal() {
        let arena = Bump::new();
        let ctx = ReportContext::new(&arena);
        let mut app = app_with_error_modal();
        golden("render_error_modal").assert(&render(&mut app, &ctx));
    }

    #[test]
    fn render_error_modal_scrolled_to_bottom() {
        let arena = Bump::new();
        let ctx = ReportContext::new(&arena);
        let mut app = app_with_error_modal();
        // The first frame is what teaches the popup its viewport height.
        render(&mut app, &ctx);
        app.update(Message::OverlayScroll(ScrollDelta::Bottom));
        golden("render_error_modal_scrolled_to_bottom").assert(&render(&mut app, &ctx));
    }
}
