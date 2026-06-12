//! Pure rendering functions for the review TUI.
//!
//! Layout: title bar, transaction queue table, preview pane showing the
//! selected transaction as Ledger text, and a footer that doubles as the
//! account prompt input line. Candidates pop up above the footer while the
//! prompt is active; confirmation overlays are centered popups.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
};
use unicode_width::UnicodeWidthStr;

use crate::import::single_entry::ReviewKind;

use super::app::{Overlay, ReviewApp, ReviewItem, Status};

const FOOTER_HINT_QUEUE: &str =
    " ↑/↓ move · a accept · e/Enter account · s skip · w write&quit · q abort ";
const FOOTER_HINT_PROMPT: &str = " Tab complete · Enter submit · Esc cancel ";
/// Maximum number of autocomplete candidates shown at once.
const PROMPT_LIST_HEIGHT: u16 = 8;
/// Preview pane height including its borders.
const PREVIEW_HEIGHT: u16 = 10;

/// Renders a frame for the given app state.
pub fn draw(frame: &mut Frame, app: &mut ReviewApp) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(1),              // title bar
        Constraint::Min(3),                 // queue table
        Constraint::Length(PREVIEW_HEIGHT), // preview pane
        Constraint::Length(1),              // footer hint / prompt input
    ])
    .split(area);

    draw_title(frame, layout[0], app);
    draw_queue(frame, layout[1], app);
    draw_preview(frame, layout[2], app);

    if app.prompt.is_some() {
        draw_prompt(frame, layout[3], app);
    } else {
        draw_footer(frame, layout[3], FOOTER_HINT_QUEUE);
    }

    if let Some(overlay) = app.overlay {
        draw_overlay(frame, area, overlay);
    }
}

fn draw_title(frame: &mut Frame, area: Rect, app: &ReviewApp) {
    let title = format!(
        " okane import — {} → {} — reviewed {}/{} ",
        app.source_display,
        app.output_display,
        app.reviewed_count(),
        app.items.len(),
    );
    let paragraph =
        Paragraph::new(Line::from(title)).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(paragraph, area);
}

fn draw_queue(frame: &mut Frame, area: Rect, app: &mut ReviewApp) {
    let block = Block::default().borders(Borders::ALL).title(" Transactions ");
    let inner = block.inner(area);
    app.nav.viewport_height = inner.height.saturating_sub(1);

    if app.items.is_empty() {
        let msg = Paragraph::new("No transactions to review")
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(" "),
        Cell::from("Date"),
        Cell::from("Payee"),
        Cell::from("Amount"),
        Cell::from("Kind"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let amount_width = app
        .items
        .iter()
        .map(|item| display_width(&item.amount))
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .clamp(10, u16::MAX as usize) as u16;

    let rows: Vec<Row> = app.items.iter().map(make_queue_row).collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(1),  // status mark
            Constraint::Length(10), // YYYY-MM-DD
            Constraint::Min(10),    // payee, takes remaining
            Constraint::Length(amount_width),
            Constraint::Length(7), // longest kind label
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .block(block);

    frame.render_stateful_widget(table, area, &mut app.nav.table_state);
}

fn make_queue_row(item: &ReviewItem) -> Row<'_> {
    let mark = match item.status {
        Status::Todo => " ",
        Status::Accepted => "✓",
        Status::Skipped => "-",
    };
    let kind = match item.kind {
        ReviewKind::Auto => "auto",
        ReviewKind::Pending => "pending",
        ReviewKind::Unknown => "unknown",
    };
    let style = match item.status {
        // Items still waiting for a decision stand out.
        Status::Todo => Style::default().add_modifier(Modifier::BOLD),
        Status::Accepted => Style::default(),
        Status::Skipped => Style::default().add_modifier(Modifier::DIM),
    };
    Row::new(vec![
        Cell::from(mark),
        Cell::from(item.date.to_string()),
        Cell::from(item.payee.as_str()),
        Cell::from(Line::from(item.amount.as_str()).alignment(Alignment::Right)),
        Cell::from(kind),
    ])
    .style(style)
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &ReviewApp) {
    let block = Block::default().borders(Borders::ALL).title(" Preview ");
    let text = match app.selected_index() {
        Some(idx) => app.items[idx].preview.as_str(),
        None => "",
    };
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, hint: &str) {
    let footer = Paragraph::new(hint).style(Style::default().add_modifier(Modifier::DIM));
    frame.render_widget(footer, area);
}

/// Renders the account input line in the footer area and the candidate list
/// popup directly above it.
fn draw_prompt(frame: &mut Frame, footer_area: Rect, app: &ReviewApp) {
    let prompt = app.prompt.as_ref().expect("prompt must be active");

    let label = " Account> ";
    let input_line = Line::from(vec![
        ratatui::text::Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw(prompt.input.as_str()),
    ]);
    frame.render_widget(Paragraph::new(input_line), footer_area);
    // Right-aligned hint, only when it does not collide with the input.
    let used = display_width(label) + display_width(&prompt.input);
    let hint_width = display_width(FOOTER_HINT_PROMPT);
    if used + hint_width < footer_area.width as usize {
        let hint_area = Rect {
            x: footer_area.right() - hint_width as u16,
            width: hint_width as u16,
            ..footer_area
        };
        draw_footer(frame, hint_area, FOOTER_HINT_PROMPT);
    }
    frame.set_cursor_position(Position {
        x: footer_area.x + used.min(footer_area.width.saturating_sub(1) as usize) as u16,
        y: footer_area.y,
    });

    if prompt.matches.is_empty() {
        return;
    }
    let height = (prompt.matches.len() as u16).min(PROMPT_LIST_HEIGHT) + 2;
    let width = prompt
        .matches
        .iter()
        .map(|&i| display_width(&app.accounts[i]))
        .max()
        .unwrap_or(0)
        .saturating_add(4)
        .max(20)
        // The terminal can be narrower than the 20-column minimum.
        .min(frame.area().width as usize) as u16;
    let popup = Rect {
        x: footer_area.x,
        y: footer_area.y.saturating_sub(height),
        width,
        height,
    };
    frame.render_widget(Clear, popup);
    let items: Vec<ListItem> = prompt
        .matches
        .iter()
        .map(|&i| ListItem::new(app.accounts[i].as_str()))
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Accounts "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default().with_selected(Some(prompt.selected));
    frame.render_stateful_widget(list, popup, &mut state);
}

/// Centered modal asking the user to confirm finishing or aborting.
fn draw_overlay(frame: &mut Frame, area: Rect, overlay: Overlay) {
    let (title, body) = match overlay {
        Overlay::WriteConfirm { unreviewed } => (
            " Write? ",
            format!("{unreviewed} unreviewed transaction(s) will be written as-is. Write? (y/n)"),
        ),
        Overlay::AbortConfirm => (
            " Abort? ",
            "Abort the import? Nothing will be written. (y/n)".to_string(),
        ),
    };
    let popup = centered_rect(area, (display_width(&body) + 4).min(60) as u16, 5);
    // Clear first so the popup paints over the queue cleanly.
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().add_modifier(Modifier::BOLD));
    let paragraph = Paragraph::new(vec![
        Line::from(""),
        Line::from(body).alignment(Alignment::Center),
    ])
    .wrap(ratatui::widgets::Wrap { trim: true })
    .block(block);
    frame.render_widget(paragraph, popup);
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width_cjk(s)
}

/// Returns a sub-rect of `area` of the given size, centered.
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::NaiveDate;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Position;

    use crate::import::single_entry::ReviewKind;
    use crate::ui::import::app::{Message, ReviewApp, ReviewItem};

    use super::draw;

    fn make_item(kind: ReviewKind, payee: &str, amount: &str) -> ReviewItem {
        ReviewItem::new(
            kind,
            format!("2024-01-02 {payee}\n    Expenses:Food    {amount}\n    Assets:Bank\n"),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            payee.to_string(),
            amount.to_string(),
        )
    }

    fn make_app() -> ReviewApp {
        ReviewApp::new(
            "import.csv".to_string(),
            "out.ledger".to_string(),
            vec![
                make_item(ReviewKind::Auto, "Coffee Shop", "10.00 CHF"),
                make_item(ReviewKind::Pending, "Supermarket", "52.30 CHF"),
                make_item(ReviewKind::Unknown, "Mystery Store", "99.00 CHF"),
            ],
            vec![
                "Assets:Bank".to_string(),
                "Expenses:Food".to_string(),
                "Expenses:Groceries".to_string(),
            ],
        )
    }

    /// Renders `app` into an 80×24 headless terminal and returns the plain text.
    fn render(app: &mut ReviewApp) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
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
            .join("../testdata/import/ui")
            .join(format!("{name}.txt"));
        okane_golden::Golden::new(path).unwrap()
    }

    #[test]
    fn render_initial_state() {
        let mut app = make_app();
        golden("render_initial_state").assert(&render(&mut app));
    }

    #[test]
    fn render_prompt_empty_input() {
        let mut app = make_app();
        // Move to the Unknown item and open the account prompt.
        app.update(Message::MoveDown);
        app.update(Message::OpenPrompt);
        golden("render_prompt_empty_input").assert(&render(&mut app));
    }

    #[test]
    fn render_prompt_filtered() {
        let mut app = make_app();
        app.update(Message::MoveDown);
        app.update(Message::OpenPrompt);
        for c in "exp".chars() {
            app.update(Message::PromptInput(c));
        }
        golden("render_prompt_filtered").assert(&render(&mut app));
    }

    #[test]
    fn render_write_confirm_overlay() {
        let mut app = make_app();
        app.update(Message::RequestWrite);
        golden("render_write_confirm_overlay").assert(&render(&mut app));
    }

    #[test]
    fn render_abort_confirm_overlay() {
        let mut app = make_app();
        app.update(Message::RequestAbort);
        golden("render_abort_confirm_overlay").assert(&render(&mut app));
    }
}
