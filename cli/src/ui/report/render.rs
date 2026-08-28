//! Pure rendering functions for the TUI.
//!
//! Both report tables draw **one row per line**: an account whose balance spans
//! six commodities occupies six table rows rather than one six-line row. That
//! is what makes every line reachable — a row taller than the body used to be
//! clipped silently, with no way to scroll to what it hid — and
//! [`crate::ui::table::LineIndex`] translates those rows back into the accounts
//! and entries the rest of the UI reasons about.
//!
//! Exploding the rows moves the account label onto the first row of its block,
//! which leaves two things to say. Both are applied by [`build_body`], which
//! knows the exact window it drew:
//!
//! - **Sticky account.** When the body's top row is a continuation, the account
//!   it belongs to has scrolled off; its label is redrawn there in
//!   [`STICKY_COLOR`]. Colour alone, with no prefix, so the text still sits in
//!   the column it belongs to — the register's date column is exactly
//!   `YYYY-MM-DD` wide and a prefix would push the date out of it.
//! - **`+N more`.** An account cut by the top or bottom edge of the body says
//!   how many of its lines the reader cannot see, in place of the amount on the
//!   edge row itself — which is one of them, since the marker takes its place.
//!
//! The popup pattern follows
//! <https://ratatui.rs/recipes/layout/center-a-widget/>:
//! render a `Clear` widget over a centered rect, then render the popup
//! contents on top.

use std::cmp::{max, min};

use ansi_to_tui::IntoText;
use okane_core::report::{Amount, ReportContext};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::app::{App, Screen};
use super::balance::{BalanceRow, BalanceView, DisplayMode};
use super::form::{Field, OptionsForm};
use super::overlay::{Overlay, TextPopup};
use super::register::RegisterView;
use super::search::{Search, SearchDirection, SearchMode, SearchPhase};
use crate::ui::table::TableNav;

/// The one binding the status bar still spells out: the whole point of the
/// help popup is that the rest of them no longer have to fit on this line.
const STATUS_HELP_HINT: &str = " ? help ";
const ERROR_POPUP_HINT: &str =
    " ↑/↓ scroll · PgUp/PgDn page · g/G top/end · r reload · Esc/Enter close · q quit ";
const HELP_POPUP_HINT: &str = " ↑/↓ scroll · PgUp/PgDn page · Esc/q close ";
const OPTIONS_POPUP_HINT: &str =
    " Tab/↑↓ field · space toggle · C-u clear · Enter apply · Esc cancel ";

/// Columns the form keeps for a value, wide enough for a date, a commodity and
/// a short path — and, being fixed, wide enough that the highlighted row reads
/// as the input box it is even when what is typed into it is short.
const OPTIONS_VALUE_WIDTH: usize = 30;

/// Colour that marks a redrawn (sticky) account label, so it never reads as a
/// label the row itself owns.
const STICKY_COLOR: Color = Color::Cyan;

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
    let status = status_text(app);
    match &mut app.screen {
        Screen::Balance => {
            draw_balance_body(frame, layout[1], &mut app.balance, ctx);
            match (&app.error_toast, &app.balance.search) {
                (Some(msg), _) => draw_error(frame, layout[2], msg),
                (None, Some(search)) => draw_search_bar(frame, layout[2], search),
                (None, None) => draw_status(frame, layout[2], &status),
            }
        }
        Screen::Register(view) => {
            draw_register_body(frame, layout[1], view, ctx);
            match &app.error_toast {
                Some(msg) => draw_error(frame, layout[2], msg),
                None => draw_status(frame, layout[2], &status),
            }
        }
    }

    match &mut app.overlay {
        Some(Overlay::QuitConfirm) => draw_quit_confirm(frame, area),
        Some(Overlay::Error(popup)) => draw_text_popup(frame, area, popup, PopupKind::Error),
        Some(Overlay::Help(popup)) => draw_text_popup(frame, area, popup, PopupKind::Help),
        Some(Overlay::Options(form)) => draw_options_form(frame, area, form),
        None => {}
    }
}

fn draw_title(frame: &mut Frame, area: Rect, app: &App<'_>) {
    let title = match &app.screen {
        Screen::Balance => format!(" okane ui — {} ", app.source_display),
        Screen::Register(view) => format!(
            " okane ui — {} — register: {} ",
            app.source_display,
            view.title()
        ),
    };
    let paragraph =
        Paragraph::new(Line::from(title)).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(paragraph, area);
}

/// One physical table row — exactly one terminal line.
#[derive(Debug, PartialEq, Eq)]
struct LineRow {
    /// Leading columns: the account, or the date and payee. Blank on a
    /// continuation line, except where the sticky overlay refills them.
    head: Vec<String>,
    /// One entry per amount column: the text on this line, or `None` where the
    /// column has no line here — a register entry's Amount is one line however
    /// tall its Total is, and marking the shorter one would be a lie.
    cells: Vec<Option<String>>,
    /// This row carries a search-matched account's own label.
    is_match: bool,
    /// [`Self::head`] was refilled by the sticky overlay rather than owned.
    sticky: bool,
}

/// The visible rows of a table body, with the sticky account label and the
/// `+N more` cut markers already applied.
struct Body {
    rows: Vec<LineRow>,
    /// The selection, as an index into [`Self::rows`].
    selected: usize,
}

/// Builds the table rows for `nav`'s current window.
///
/// `head_of` and `columns_of` are called once per *item* in the window rather
/// than once per row, so a multi-commodity account formats its amounts once
/// however many lines it spans. Neither is called for anything off screen,
/// which is what keeps per-frame work proportional to the viewport rather than
/// to the row count.
fn build_body(
    nav: &mut TableNav,
    head_of: impl Fn(usize) -> Vec<String>,
    columns_of: impl Fn(usize) -> Vec<Vec<String>>,
    is_match: impl Fn(usize) -> bool,
) -> Body {
    let (offset, end) = nav.visible_window();
    let mut rows: Vec<LineRow> = Vec::with_capacity(end - offset);
    // Set only when the top item's own first row is above the window: its head,
    // which the sticky overlay redraws on the body's top line, and the lines it
    // hides up there.
    let mut cut_above: Option<(Vec<String>, u16)> = None;
    // The bottom item's line at the bottom edge, with how many lines each of
    // its columns holds — between them, what it hides below.
    let mut bottom_edge: Option<(u16, Vec<usize>)> = None;

    for (nth, (item, window)) in nav.lines().items_in(offset..end).enumerate() {
        let head = head_of(item);
        let columns = columns_of(item);
        let matched = is_match(item);
        if nth == 0 && window.start > 0 {
            // This item's own label went off screen with the lines above it.
            cut_above = Some((head.clone(), window.start));
        }
        bottom_edge = Some((window.end - 1, columns.iter().map(Vec::len).collect()));
        for line in window {
            rows.push(LineRow {
                head: match line {
                    0 => head.clone(),
                    // A continuation line owns no label; the sticky overlay
                    // below refills the top one when the item's own is gone.
                    _ => vec![String::new(); head.len()],
                },
                cells: columns
                    .iter()
                    .map(|col| col.get(usize::from(line)).cloned())
                    .collect(),
                is_match: matched && line == 0,
                sticky: false,
            });
        }
    }

    // No *row* is ever clipped now, but an item's block still runs past the top
    // and bottom of the body, which is the same news the marker has always
    // carried. Above the top row every column that reaches it hid the same
    // lines; below the bottom one each column ran out where it ran out.
    if let Some((head, above)) = cut_above
        && let Some(top) = rows.first_mut()
    {
        top.head = head;
        top.sticky = true;
        mark_cut(top, |_| usize::from(above), "above");
    }
    if let Some((edge, lengths)) = bottom_edge
        && let Some(bottom) = rows.last_mut()
    {
        let below = usize::from(edge) + 1;
        mark_cut(
            bottom,
            |column| lengths[column].saturating_sub(below),
            "more",
        );
    }

    Body {
        selected: nav.selected_row().saturating_sub(offset),
        rows,
    }
}

/// Replaces the amounts on a cut edge row with `+{n} {suffix}`, asking `beyond`
/// per column how many of that column's lines lie past the edge: how much a
/// column hides is its own business — a register entry's Amount is one line
/// however far past the edge its Total runs — and a column that ends on the
/// edge keeps its value, since there is nothing past it to warn about.
///
/// The count is `beyond + 1`, because the marker also spends the line it is
/// written on: the value that was there is one the reader cannot see either.
/// Spending the edge line rather than adding one keeps the body exactly as tall
/// as the viewport, which is what lets the marker be decided after the window
/// is fixed rather than as part of choosing it.
fn mark_cut(row: &mut LineRow, beyond: impl Fn(usize) -> usize, suffix: &str) {
    for (column, cell) in row.cells.iter_mut().enumerate() {
        let beyond = beyond(column);
        if beyond > 0 && cell.is_some() {
            *cell = Some(format!("+{} {suffix}", beyond + 1));
        }
    }
}

/// Everything about a table body that is not its data: what to say when there
/// is none, the header labels, and the column constraints. One per screen,
/// built where the column widths are known.
struct TableChrome<'a> {
    empty_message: &'a str,
    headers: &'a [&'a str],
    constraints: Vec<Constraint>,
}

/// Draws one table body: the bordered block, the window `nav` selects, and the
/// rows [`build_body`] makes of it.
///
/// Both screens differ only in their [`TableChrome`] and in what the three
/// closures read, so the framing — the empty and too-short notices, the
/// viewport height the nav needs, and rendering only the chosen window so
/// ratatui never scrolls on its own — lives here once.
fn draw_table_body(
    frame: &mut Frame,
    area: Rect,
    nav: &mut TableNav,
    chrome: TableChrome<'_>,
    head_of: impl Fn(usize) -> Vec<String>,
    columns_of: impl Fn(usize) -> Vec<Vec<String>>,
    is_match: impl Fn(usize) -> bool,
) {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    // The header takes the first line of the body; the rest is the window.
    nav.viewport_height = inner.height.saturating_sub(1);

    if nav.is_empty() {
        let msg = Paragraph::new(chrome.empty_message)
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(msg, area);
        return;
    }
    if draw_body_too_short(frame, area, &block) {
        return;
    }

    let body = build_body(nav, head_of, columns_of, is_match);
    let header = Row::new(chrome.headers.iter().copied().map(Cell::from))
        .style(Style::default().add_modifier(Modifier::BOLD));
    let table = Table::new(body.rows.iter().map(to_table_row), chrome.constraints)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .block(block);

    // The window was already chosen by `TableNav::visible_window`, and the
    // marker and sticky-label decisions depend on it being the one actually
    // drawn, so hand ratatui a state that keeps it.
    let mut state = TableState::default()
        .with_offset(0)
        .with_selected(Some(body.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_balance_body<'ctx>(
    frame: &mut Frame,
    area: Rect,
    view: &mut BalanceView<'ctx>,
    ctx: &ReportContext<'ctx>,
) {
    // Split the borrows: the row closures below read `rows` while
    // `draw_table_body` drives `nav`.
    let BalanceView {
        rows,
        nav,
        search,
        mode,
        amount_width,
        ..
    } = view;
    let (rows, mode) = (&*rows, *mode);
    let matches = search
        .as_ref()
        .and_then(|s| s.matches.as_ref())
        .and_then(|r| r.as_ref().ok());

    // The column is one width for the whole table — the one thing that has to
    // look at every row — so it is computed once per set of rows and cached;
    // the rows themselves are built for the visible window only.
    let amount_width = *amount_width.get_or_insert_with(|| {
        compute_amount_width(rows.iter().map(|row| format_amount_lines(&row.amount, ctx)))
    });

    draw_table_body(
        frame,
        area,
        nav,
        TableChrome {
            empty_message: "No balances to display",
            headers: &["Account", "Amount"],
            constraints: vec![Constraint::Min(10), Constraint::Length(amount_width)],
        },
        |i| vec![account_cell_text(&rows[i], mode)],
        |i| vec![format_amount_lines(&rows[i].amount, ctx)],
        |i| matches.is_some_and(|m| m.contains_row(i)),
    );
}

fn draw_register_body<'ctx>(
    frame: &mut Frame,
    area: Rect,
    view: &mut RegisterView<'ctx>,
    ctx: &ReportContext<'ctx>,
) {
    // Split the borrows: the row closures below read `rows` while
    // `draw_table_body` drives `nav`.
    let RegisterView {
        rows,
        nav,
        col_widths,
        ..
    } = view;
    let rows = &*rows;

    // The amount/total column widths are fixed for the life of the view, so
    // compute them once (scanning all rows) and cache them.
    let (amount_width, total_width) = *col_widths.get_or_insert_with(|| {
        (
            compute_amount_width(rows.iter().map(|row| format_amount_lines(&row.amount, ctx))),
            compute_amount_width(rows.iter().map(|row| format_amount_lines(&row.total, ctx))),
        )
    });

    draw_table_body(
        frame,
        area,
        nav,
        TableChrome {
            empty_message: "No register entries to display",
            headers: &["Date", "Payee", "Amount", "Total"],
            constraints: vec![
                Constraint::Length(10), // YYYY-MM-DD
                Constraint::Min(10),    // payee, takes remaining
                Constraint::Length(amount_width),
                Constraint::Length(total_width),
            ],
        },
        |i| vec![rows[i].date.to_string(), rows[i].payee.clone()],
        |i| {
            vec![
                format_amount_lines(&rows[i].amount, ctx),
                format_amount_lines(&rows[i].total, ctx),
            ]
        },
        |_| false,
    );
}

/// A body with no room for a row after its header draws nothing at all, however
/// short the rows are — and now that no row is ever taller than one line, there
/// is nothing left to cap. Say so instead of showing an empty box, which reads
/// as "no data". The border goes too: at this size it *is* the body.
fn draw_body_too_short(frame: &mut Frame, area: Rect, block: &Block) -> bool {
    if block.inner(area).height >= 2 {
        return false;
    }
    frame.render_widget(
        Paragraph::new("terminal too short")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red)),
        area,
    );
    true
}

/// Where the selection sits in the screen's items — `Account 3/57`, or
/// `Register 12/40` — counted in accounts and register entries rather than in
/// the table rows a multi-commodity one spans, which is what the reader is
/// moving through with `J`/`K`.
///
/// Followed by the query the numbers were computed under, where it is not the
/// plain one: a converted or date-bounded report looks exactly like a full one,
/// and `.` can change it mid-session, so the line that says where you are also
/// says what you are looking at.
fn status_text(app: &App<'_>) -> String {
    let (label, nav) = match &app.screen {
        Screen::Balance => ("Account", &app.balance.nav),
        Screen::Register(view) => ("Register", &view.nav),
    };
    // The position is 1-based; an empty table has no position at all and says
    // so with a `0/0` rather than pretending to sit on a first item.
    let position = nav.selected_item().map_or(0, |item| item + 1);
    let count = nav.item_count();
    match app.query.options.summary() {
        Some(query) => format!(" {label} {position}/{count} · {query} "),
        None => format!(" {label} {position}/{count} "),
    }
}

/// The status bar: where the selection is, and — at the far end, where it
/// survives a narrow terminal least — the one key that shows all the others.
fn draw_status(frame: &mut Frame, area: Rect, status: &str) {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let layout = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(saturating_u16(layout_width(STATUS_HELP_HINT))),
    ])
    .split(area);
    frame.render_widget(Paragraph::new(status), layout[0]);
    frame.render_widget(Paragraph::new(STATUS_HELP_HINT).style(dim), layout[1]);
}

/// Renders a transient error notice (e.g. a failed reload) in the footer
/// slot. Takes priority over the key hint and the search bar; cleared by
/// [`App::update`] on the next key press.
fn draw_error(frame: &mut Frame, area: Rect, message: &str) {
    let footer = Paragraph::new(format!(" {message} ")).style(Style::default().fg(Color::Red));
    frame.render_widget(footer, area);
}

/// Renders the balance search bar in the footer slot: a prompt + the typed
/// pattern (red when it yields no usable regex) plus a dim hint suffix. While editing it
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
        (Some(err), _) => Span::styled(
            format!("  {}", err.label()),
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

/// The query-options form (`.`): one row per option, the focused one drawn as
/// the input box it is, and — where the last submit was refused — the reason
/// under them in place of the key hint.
///
/// Unlike the text popups this is sized to its content in both directions and
/// never scrolls: it has five rows, and a terminal too small for five rows is
/// too small for the report behind them.
fn draw_options_form(frame: &mut Frame, area: Rect, form: &OptionsForm) {
    let label_width = form
        .fields()
        .iter()
        .map(|field| layout_width(field.label()))
        .max()
        .unwrap_or(0);
    // The rows are two columns and their gutters; the footer is one long line.
    let body_width = label_width + OPTIONS_VALUE_WIDTH + 4;
    let width = max(body_width, layout_width(OPTIONS_POPUP_HINT)) + 2; // borders
    let height = form.fields().len() + 3; // borders + the footer line
    let rect = centered_rect(area, saturating_u16(width), saturating_u16(height));
    // Clear first so the popup paints over the table cleanly.
    frame.render_widget(Clear, rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Query options ")
        .style(Style::default().add_modifier(Modifier::BOLD));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);
    let rows: Vec<Line> = form
        .fields()
        .iter()
        .enumerate()
        .map(|(i, field)| options_row(field, label_width, i == form.focus()))
        .collect();
    frame.render_widget(
        Paragraph::new(rows).style(Style::default().remove_modifier(Modifier::BOLD)),
        layout[0],
    );

    let footer = match form.error() {
        Some(err) => Paragraph::new(format!(" {err} ")).style(Style::default().fg(Color::Red)),
        None => {
            Paragraph::new(OPTIONS_POPUP_HINT).style(Style::default().add_modifier(Modifier::DIM))
        }
    };
    frame.render_widget(footer, layout[1]);

    // The cursor sits after the last typed character of the focused row, which
    // is also what tells a value apart from the placeholder standing in for it.
    // A value too long for the box is scrolled, so the cursor stops at its end.
    let field = &form.fields()[form.focus()];
    if field.is_text() {
        let typed = min(layout_width(field.text()), options_value_columns());
        let x = layout[0].x + saturating_u16(label_width + 3 + typed);
        let x = min(x, layout[0].x + layout[0].width.saturating_sub(1));
        frame.set_cursor_position((x, layout[0].y + saturating_u16(form.focus())));
    }
}

/// Columns of text the value box holds, inside its one-space padding.
fn options_value_columns() -> usize {
    OPTIONS_VALUE_WIDTH.saturating_sub(2)
}

/// One row of the form: `--flag   value`, with the value drawn as a
/// fixed-width box so the focused row's highlight has a shape, and an unset
/// value replaced by its dim placeholder.
fn options_row(field: &Field, label_width: usize, focused: bool) -> Line<'static> {
    let label = format!(
        " {}{}  ",
        field.label(),
        " ".repeat(label_width.saturating_sub(layout_width(field.label())))
    );
    let (text, dim) = match field.text() {
        "" => (field.placeholder(), true),
        text => (text, false),
    };
    // A path longer than the box scrolls: the tail is where the typing is
    // happening, and clipping it at the border would hide exactly that.
    let text = tail(text, options_value_columns());
    let value = format!(
        " {text}{} ",
        " ".repeat(options_value_columns().saturating_sub(layout_width(text)))
    );
    let mut value_style = Style::default();
    if dim {
        value_style = value_style.add_modifier(Modifier::DIM);
    }
    if focused {
        value_style = value_style.add_modifier(Modifier::REVERSED);
    }
    Line::from(vec![Span::raw(label), Span::styled(value, value_style)])
}

/// The last `columns` display columns of `text`, cut on a character boundary.
/// The whole of it when it already fits.
fn tail(text: &str, columns: usize) -> &str {
    let mut width = 0;
    let mut start = text.len();
    for (i, c) in text.char_indices().rev() {
        width += UnicodeWidthChar::width(c).unwrap_or(0);
        if width > columns {
            break;
        }
        start = i;
    }
    &text[start..]
}

/// What a [`TextPopup`] is showing: it decides the frame color and the hint
/// under the body, and nothing else — the two scroll and clip alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupKind {
    /// A failure the user must acknowledge, e.g. a reload that could not parse
    /// the ledger.
    Error,
    /// The key bindings of the screen underneath.
    Help,
}

impl PopupKind {
    fn style(self) -> Style {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        match self {
            PopupKind::Error => bold.fg(Color::Red),
            PopupKind::Help => bold,
        }
    }

    fn hint(self) -> &'static str {
        match self {
            PopupKind::Error => ERROR_POPUP_HINT,
            PopupKind::Help => HELP_POPUP_HINT,
        }
    }

    /// Where the modal goes. An error report is text of any size, so it takes
    /// a fixed slice of the screen; the help is a list of known length, and a
    /// frame bigger than it holds is only more of the screen hidden for
    /// nothing.
    fn rect(self, area: Rect, popup: &TextPopup) -> Rect {
        match self {
            PopupKind::Error => popup_rect(area),
            PopupKind::Help => content_rect(area, popup, self.hint()),
        }
    }
}

/// Near-full-screen modal showing a body of text in full.
///
/// The body is deliberately *not* wrapped: annotate-snippets output aligns a
/// line-number gutter, the source line and a caret row by column, and soft
/// wrapping would move the carets away from what they point at (the help's key
/// column is aligned the same way). Long lines are clipped instead. That also
/// keeps `lines.len()` the rendered line count, so [`TextPopup::scroll`] can
/// clamp exactly.
fn draw_text_popup(frame: &mut Frame, area: Rect, popup: &mut TextPopup, kind: PopupKind) {
    let rect = kind.rect(area, popup);
    // Clear first so the popup paints over the table cleanly.
    frame.render_widget(Clear, rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(popup.title.as_str())
        .style(kind.style());
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);
    // The body height is only known here; keep it fresh for the scroll math,
    // the same way the tables refresh `TableNav::viewport_height`.
    popup.viewport_height = layout[0].height;
    popup.clamp();

    let joined = popup.lines.join("\n");
    let body = joined.into_text().unwrap_or_else(|_| Text::from(joined));
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
    frame.render_widget(Paragraph::new(kind.hint()).style(dim), footer[0]);
    frame.render_widget(Paragraph::new(position).style(dim), footer[1]);
}

/// Columns kept for the position counter beside the hint, enough for
/// ` 99/99 ` — the widest one a help of any plausible length produces.
const POSITION_WIDTH: usize = 8;

/// A popup exactly as big as the text it holds — its widest line, or its
/// footer where that is wider — clamped to the screen, which is where a body
/// too tall to fit starts scrolling instead.
fn content_rect(area: Rect, popup: &TextPopup, hint: &str) -> Rect {
    let body = popup
        .lines
        .iter()
        .map(|line| layout_width(line))
        .max()
        .unwrap_or(0);
    let width = max(body, layout_width(hint) + POSITION_WIDTH) + 2; // borders
    let height = popup.lines.len() + 3; // borders + the hint line
    centered_rect(area, saturating_u16(width), saturating_u16(height))
}

fn saturating_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// Four fifths of the screen, with a floor so tiny terminals still show
/// something usable.
fn popup_rect(area: Rect) -> Rect {
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
///
/// Must agree with [`amount_line_count`]; see there.
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

/// Lines [`format_amount_lines`] will produce for `amount` (always `>= 1`),
/// without formatting them.
///
/// This is what the screens size their [`LineIndex`](crate::ui::table::LineIndex)
/// with, so it decides how many table rows an account or entry gets. It lives
/// next to the function it predicts because the two must agree exactly: were
/// the count to fall short, the extra lines would get no row and become
/// unreachable — the very thing exploding the rows set out to fix — and were it
/// to run over, the table would draw blank rows nothing can fill.
pub(super) fn amount_line_count(amount: &Amount<'_>) -> u16 {
    let n = amount.iter().count();
    n.clamp(1, u16::MAX as usize) as u16
}

/// Turns one [`LineRow`] into a ratatui row: the head columns as text, the
/// amount columns right-aligned.
fn to_table_row(row: &LineRow) -> Row<'_> {
    let head_style = match (row.sticky, row.is_match) {
        (true, _) => Style::default().fg(STICKY_COLOR),
        (false, true) => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        (false, false) => Style::default(),
    };
    Row::new(
        row.head
            .iter()
            .map(|text| Cell::from(text.as_str()).style(head_style))
            .chain(
                row.cells
                    .iter()
                    .map(|line| Cell::from(amount_text(line.as_deref()))),
            ),
    )
}

/// Account-column text for a balance row: the full name in flat mode, or the
/// depth-indented leaf label with a fold marker in the gutter in tree mode.
fn account_cell_text(row: &BalanceRow<'_>, mode: DisplayMode) -> String {
    match mode {
        DisplayMode::Flat => row.label.to_owned(),
        DisplayMode::Tree => {
            let indent = "  ".repeat(usize::from(row.depth.saturating_sub(1)));
            let marker = if row.has_children {
                if row.folded { "▶" } else { "▼" }
            } else {
                " "
            };
            format!("{indent}{marker} {}", row.label)
        }
    }
}

/// One right-aligned amount line, or nothing where the column has run out of
/// them.
fn amount_text(line: Option<&str>) -> Text<'_> {
    match line {
        Some(text) => Text::from(Line::from(text).alignment(Alignment::Right)),
        None => Text::default(),
    }
}

/// Returns the display width (in columns) needed for an amount column, given
/// the per-row formatted lines. Accepts anything iterable-of-iterable-of-str,
/// so callers can pass either a materialized `&[Vec<String>]` (the balance
/// screen, which needs the lines again to build rows) or a lazy iterator of
/// `Vec<String>` (the register, which only needs the width).
fn compute_amount_width<Rows, Lines, S>(formatted: Rows) -> u16
where
    Rows: IntoIterator<Item = Lines>,
    Lines: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let max = formatted
        .into_iter()
        .flatten()
        .map(|s| display_width(s.as_ref()))
        .max()
        .unwrap_or(0);
    // Add a 1-column right padding so the value never abuts the border.
    max.saturating_add(1).clamp(10, u16::MAX as usize) as u16
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width_cjk(s)
}

/// Plain (non-CJK) width: what ratatui itself lays glyphs out with, and so the
/// measure popup geometry has to use — [`display_width`] deliberately differs,
/// since an amount column is sized for the terminals that draw East-Asian
/// *ambiguous* glyphs wide.
fn layout_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{Path, PathBuf};

    use bumpalo::Bump;
    use okane_core::report::query::DateRange;
    use okane_core::report::{Amount, ReportContext};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Position;
    use rstest::rstest;
    use rust_decimal_macros::dec;

    use super::super::app::Message;
    use super::super::balance::BalanceMessage;
    use super::super::form::FormMessage;
    use super::super::overlay::ScrollDelta;
    use super::super::register::{RegisterMessage, RegisterQueryTemplate};
    use super::super::testing::{query_state, template};
    use crate::ui::table::NavCommand;

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

    /// The status bar counts *accounts*, not the table rows a multi-commodity
    /// account spans: walking a line down inside one leaves the count alone,
    /// while stepping to the next account advances it.
    #[test]
    fn status_counts_accounts_rather_than_rows() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        render(&mut app, &ctx); // the first frame teaches the nav its viewport
        let total = app.balance.nav.item_count();

        assert_eq!(status_text(&app), format!(" Account 1/{total} "));
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
        assert_eq!(status_text(&app), format!(" Account 1/{total} "));
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::NextItem)));
        assert_eq!(status_text(&app), format!(" Account 2/{total} "));
    }

    #[test]
    fn status_of_an_empty_screen_has_no_position() {
        let app = App::new("test.ledger".to_owned(), Vec::new(), query_state());
        assert_eq!(status_text(&app), " Account 0/0 ");
    }

    /// The register screen says which register entry it is on instead.
    #[test]
    fn status_on_the_register_counts_entries() {
        let arena = Bump::new();
        let (ctx, mut app) = register_app_with_rows(&arena, 50);
        render(&mut app, &ctx);
        // The register opens pinned to its last entry.
        assert_eq!(status_text(&app), " Register 50/50 ");
        app.update(Message::Register(RegisterMessage::Nav(NavCommand::First)));
        assert_eq!(status_text(&app), " Register 1/50 ");
    }

    /// The help hint is the one binding the status bar still spells out.
    #[test]
    fn status_bar_keeps_the_help_hint() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        let out = render(&mut app, &ctx);
        let status = out.lines().last().expect("a status line");
        assert!(status.starts_with(" Account 1/"), "{status:?}");
        assert!(status.trim_end().ends_with("? help"), "{status:?}");
    }

    /// A one-amount-column body over items of the given line counts, with the
    /// window sized to `viewport` and the selection on row `selected`. Item `i`
    /// is headed `item{i}` and its lines read `{i}.{line}`.
    fn body_of(line_counts: &[u16], viewport: u16, selected: usize) -> Body {
        let mut nav = TableNav::with_lines(line_counts.iter().copied());
        nav.viewport_height = viewport;
        nav.select_row(selected);
        build_body(
            &mut nav,
            |i| vec![format!("item{i}")],
            |i| vec![(0..line_counts[i]).map(|l| format!("{i}.{l}")).collect()],
            |_| false,
        )
    }

    fn heads(body: &Body) -> Vec<&str> {
        body.rows.iter().map(|row| row.head[0].as_str()).collect()
    }

    fn amounts(body: &Body) -> Vec<&str> {
        body.rows
            .iter()
            .map(|row| row.cells[0].as_deref().unwrap_or(""))
            .collect()
    }

    #[test]
    fn build_body_gives_every_amount_line_its_own_row() {
        let body = body_of(&[1, 3], 10, 0);
        // Only the first row of an item carries its label.
        assert_eq!(heads(&body), ["item0", "item1", "", ""]);
        assert_eq!(amounts(&body), ["0.0", "1.0", "1.1", "1.2"]);
        assert_eq!(body.selected, 0);
    }

    /// The marker counts every line the reader cannot see, the one it is
    /// written over included: of six lines, two are readable and four are not.
    #[test]
    fn build_body_marks_the_lines_cut_below() {
        let body = body_of(&[6], 3, 0);
        assert_eq!(amounts(&body), ["0.0", "0.1", "+4 more"]);
        // Nothing is cut above, so the label is the row's own.
        assert!(!body.rows[0].sticky);
    }

    /// The follow-up problem: once the top of an account's block is off screen
    /// its name goes with it, so it is redrawn on the top line — and the same
    /// line says how many of the account's lines it is standing in front of.
    #[test]
    fn build_body_sticks_the_label_of_an_account_cut_above() {
        let body = body_of(&[1, 6], 3, 6);
        assert_eq!(heads(&body), ["item1", "", ""]);
        assert!(body.rows[0].sticky);
        assert_eq!(amounts(&body), ["+4 above", "1.4", "1.5"]);
        // The selection is the last row of the window, window-relative.
        assert_eq!(body.selected, 2);
    }

    #[test]
    fn build_body_leaves_a_fully_visible_body_unmarked() {
        let body = body_of(&[2, 2], 4, 0);
        assert_eq!(heads(&body), ["item0", "", "item1", ""]);
        assert_eq!(amounts(&body), ["0.0", "0.1", "1.0", "1.1"]);
        assert!(body.rows.iter().all(|row| !row.sticky));
    }

    /// A register entry's Amount is one line however tall its Total is, so the
    /// two columns are cut at different places — or not at all. Marking a
    /// column that hid nothing would both lie and eat a real value.
    fn register_shaped_body(amount_lines: usize, viewport: u16) -> Body {
        let mut nav = TableNav::with_lines([4]);
        nav.viewport_height = viewport;
        nav.select_row(0);
        build_body(
            &mut nav,
            |_| vec!["2024-01-01".to_owned(), "payee".to_owned()],
            |_| {
                vec![
                    (0..amount_lines).map(|l| format!("amount{l}")).collect(),
                    (0..4).map(|l| format!("total{l}")).collect(),
                ]
            },
            |_| false,
        )
    }

    #[test]
    fn build_body_leaves_a_column_with_no_line_on_the_edge_alone() {
        let body = register_shaped_body(1, 2);
        assert_eq!(
            body.rows[0].cells,
            [Some("amount0".to_owned()), Some("total0".to_owned())]
        );
        // The Amount column ran out a line ago; only the Total is still cut,
        // hiding `total1` under the marker and `total2`/`total3` below it.
        assert_eq!(body.rows[1].cells, [None, Some("+3 more".to_owned())]);
    }

    #[test]
    fn build_body_leaves_a_column_that_ends_on_the_edge_alone() {
        let body = register_shaped_body(2, 2);
        // `amount1` is the Amount column's last line: nothing is hidden behind
        // it, so it stays put while the Total says what it is still holding.
        assert_eq!(
            body.rows[1].cells,
            [Some("amount1".to_owned()), Some("+3 more".to_owned())]
        );
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

    /// The invariant the exploded rows rest on: the count the `LineIndex` is
    /// built from is exactly the number of lines the renderer then produces.
    /// A count that fell short would leave lines with no row to be drawn on —
    /// unreachable, the bug this all fixes — and one that ran over would draw
    /// rows nothing fills.
    #[test]
    fn amount_line_count_agrees_with_format_amount_lines() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let usd = ctx.commodity_store_mut().ensure("USD");
        let eur = ctx.commodity_store_mut().ensure("EUR");
        let chf = ctx.commodity_store_mut().ensure("CHF");

        let one = Amount::from_value(usd, dec!(100));
        let three =
            one.clone() + Amount::from_value(eur, dec!(200)) + Amount::from_value(chf, dec!(3));
        for amount in [Amount::zero(), one, three] {
            assert_eq!(
                usize::from(amount_line_count(&amount)),
                format_amount_lines(&amount, &ctx).len(),
                "line count and rendered lines disagree for {amount:?}"
            );
        }
    }

    /// A value longer than the form's box shows its end: that is where the
    /// cursor is and what was just typed.
    #[test]
    fn tail_keeps_the_end_of_a_long_value() {
        assert_eq!(tail("short", 10), "short");
        assert_eq!(tail("/home/user/ledger/prices.db", 10), "/prices.db");
        // Cut on a character boundary, never inside one.
        assert_eq!(tail("ドルの価格", 4), "価格");
        assert_eq!(tail("", 10), "");
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
        let r = popup_rect(area);
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
        let r = popup_rect(area);
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
            let mut x = 0;
            while x < buf.area.width {
                let sym = buf[Position { x, y }].symbol();
                s.push_str(sym);
                // A double-width (CJK) glyph already spans two columns; ratatui
                // stores it in one cell and blanks the following continuation
                // cell. Skip that placeholder so the dumped text lines up the
                // same way the real terminal draws it. This must use the plain
                // (non-CJK) width — the one ratatui used to lay the glyph out —
                // so East-Asian *ambiguous* cells (│, ─, ·, ↑) that occupy a
                // single cell are not mistaken for wide glyphs.
                x += max(UnicodeWidthStr::width(sym) as u16, 1);
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
        let mut app = App::new("test.ledger".to_owned(), Vec::new(), query_state());
        let lines = (0..40).map(|i| format!("error line {i}")).collect();
        app.overlay = Some(Overlay::Error(TextPopup::new(
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

    /// The help popup over each screen: what `?` shows, and — since the status
    /// bar is what sends the reader here — that the two are drawn together.
    #[rstest]
    #[case::balance("many_commodities.ledger", false)]
    #[case::register("many_commodities.ledger", true)]
    fn render_help_popup(#[case] fixture: &str, #[case] register: bool) {
        let arena = Bump::new();
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../testdata/report")
            .join(fixture);
        let (ctx, mut app) = if register {
            register_app(&arena, &input)
        } else {
            balance_app(&arena, &input)
        };
        app.update(Message::ShowHelp);
        let name = if register {
            "render_help_register"
        } else {
            "render_help_balance"
        };
        golden(name).assert(&render(&mut app, &ctx));
    }

    /// The query-options form over the balance screen: every option on its own
    /// row, the focused one as an input box, and the placeholders standing in
    /// for what is unset.
    #[test]
    fn render_options_form() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        app.update(Message::ShowOptions);
        golden("render_options_form").assert(&render(&mut app, &ctx));
    }

    /// A refused submit keeps the form up and replaces the key hint with the
    /// reason, so the row that is wrong is still on screen next to it.
    #[test]
    fn render_options_form_with_a_bad_date() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        app.update(Message::ShowOptions);
        app.update(Message::Form(FormMessage::FocusNext)); // --historical
        app.update(Message::Form(FormMessage::FocusNext)); // --start
        for c in "last week".chars() {
            app.update(Message::Form(FormMessage::Push(c)));
        }
        app.update(Message::Form(FormMessage::Submit));
        golden("render_options_form_error").assert(&render(&mut app, &ctx));
    }

    /// A converted, date-bounded report looks exactly like a plain one, so the
    /// status bar says which one it is.
    #[test]
    fn status_bar_names_a_non_default_query() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        assert!(!status_text(&app).contains('·'), "{}", status_text(&app));

        app.query.options.exchange = Some("CHF".to_owned());
        app.query.options.start = Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let out = render(&mut app, &ctx);
        let status = out.lines().last().expect("a status line");
        assert!(status.contains("-X CHF"), "{status:?}");
        assert!(status.contains("2024-01-01.."), "{status:?}");
        // The help hint keeps its place at the far end.
        assert!(status.trim_end().ends_with("? help"), "{status:?}");
    }

    /// The help is drawn without wrapping, so a description longer than the
    /// screen is silently cut. The popup sizes itself to its text, which means
    /// the check is that the text still fits a plain 80-column terminal.
    #[test]
    fn help_fits_an_eighty_column_terminal() {
        let mut app = App::new("test.ledger".to_owned(), Vec::new(), query_state());
        app.update(Message::ShowHelp);
        let Some(Overlay::Help(popup)) = &app.overlay else {
            panic!("expected the help overlay");
        };
        let screen = Rect::new(0, 0, 80, 24);
        let rect = content_rect(screen, popup, HELP_POPUP_HINT);
        assert!(
            rect.width < screen.width,
            "the help wants {} columns, more than an 80-column terminal has: {:#?}",
            rect.width,
            popup.lines
        );
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

    /// The point of #489: ANSI SGR codes in the error text (as produced by
    /// annotate-snippets' styled renderer) are parsed into real cell colors
    /// instead of showing up as literal escape bytes.
    #[test]
    fn render_error_modal_parses_ansi_colors() {
        let arena = Bump::new();
        let ctx = ReportContext::new(&arena);
        let mut app = App::new("test.ledger".to_owned(), Vec::new(), query_state());
        app.overlay = Some(Overlay::Error(TextPopup::new(
            " failed to load test.ledger ".to_owned(),
            vec!["\u{1b}[32mgreen\u{1b}[0m plain".to_owned()],
        )));

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &mut app, &ctx)).unwrap();
        let buf = terminal.backend().buffer().clone();

        let mut found_green = false;
        let mut found_escape_byte = false;
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[Position { x, y }];
                if cell.symbol() == "g" && cell.fg == Color::Green {
                    found_green = true;
                }
                if cell.symbol().contains('\u{1b}') {
                    found_escape_byte = true;
                }
            }
        }
        assert!(
            found_green,
            "expected a cell styled green from the parsed ANSI escape"
        );
        assert!(
            !found_escape_byte,
            "escape bytes should be consumed by the parser, not rendered as glyphs"
        );
    }
    /// Loads a `testdata/report/*.ledger` fixture from disk and returns the
    /// report context together with its processed `Ledger`, ready to query —
    /// via the same [`load::new_loader`] production path the CLI uses.
    fn session_from_file<'ctx>(
        arena: &'ctx Bump,
        input: &Path,
    ) -> (ReportContext<'ctx>, okane_core::report::query::Ledger<'ctx>) {
        use okane_core::{load, report};

        let loader = load::new_loader(input.to_path_buf());
        let mut ctx = ReportContext::new(arena);
        let ledger = report::process(&mut ctx, loader, &report::ProcessOptions::default()).unwrap();
        (ctx, ledger)
    }

    /// The file name (e.g. `multi_commodity.ledger`) shown in the title bar.
    fn source_display(input: &Path) -> String {
        input.file_name().unwrap().to_string_lossy().into_owned()
    }

    /// The golden slug for `input` under `testdata/report/ui/`, e.g.
    /// `multi_commodity.balance`.
    fn golden_name(input: &Path, report: &str) -> String {
        format!("{}.{report}", input.file_stem().unwrap().to_string_lossy())
    }

    /// The balance screen backed by `input`'s real computed balances, built the
    /// same way [`super::super::run_ui`] does: through the [`BalanceTree`] and
    /// [`App::new`], so the flat-view rows are the production rows.
    fn balance_app<'ctx>(arena: &'ctx Bump, input: &Path) -> (ReportContext<'ctx>, App<'ctx>) {
        use okane_core::report::BalanceTree;
        use okane_core::report::query::{AccountFilter, BalanceQuery};

        let (ctx, mut ledger) = session_from_file(arena, input);
        let query = BalanceQuery {
            account: AccountFilter::All,
            conversion: None,
            date_range: DateRange::default(),
        };
        let balance = ledger.balance(&ctx, &query).unwrap().into_owned();
        let tree = BalanceTree::create(&ctx, balance).unwrap().into_nodes();
        let app = App::new(source_display(input), tree, query_state());
        (ctx, app)
    }

    /// The register screen for `input`, drilled into its first account (sorted
    /// order — deterministic and present in every fixture), its rows loaded
    /// through the production query path.
    fn register_app<'ctx>(arena: &'ctx Bump, input: &Path) -> (ReportContext<'ctx>, App<'ctx>) {
        use super::super::register::RegisterScope;
        use okane_core::report::query::{AccountFilter, BalanceQuery};

        let (ctx, mut ledger) = session_from_file(arena, input);
        let query = BalanceQuery {
            account: AccountFilter::All,
            conversion: None,
            date_range: DateRange::default(),
        };
        let account = ledger
            .balance(&ctx, &query)
            .unwrap()
            .into_owned()
            .into_vec()
            .into_iter()
            .next()
            .map(|(account, _)| account)
            .expect("fixture ledger has at least one account");
        let scope = RegisterScope::Single(account);
        let rows =
            super::super::fulfill::load_register(&mut ledger, &ctx, &template(), scope).unwrap();
        let mut app = App::new(source_display(input), Vec::new(), query_state());
        app.show_register(scope, rows);
        (ctx, app)
    }

    /// The register screen for an account *subtree*
    /// (`RegisterScope::Subtree` / `AccountFilter::descendants_of`): the first
    /// parent account in tree order, with the rows of every descendant, loaded
    /// through the production query path.
    fn register_subtree_app<'ctx>(
        arena: &'ctx Bump,
        input: &Path,
    ) -> (ReportContext<'ctx>, App<'ctx>) {
        use okane_core::report::BalanceTree;
        use okane_core::report::query::{AccountFilter, BalanceQuery};

        let (ctx, mut ledger) = session_from_file(arena, input);
        let query = BalanceQuery {
            account: AccountFilter::All,
            conversion: None,
            date_range: DateRange::default(),
        };
        let balance = ledger.balance(&ctx, &query).unwrap().into_owned();
        let tree = BalanceTree::create(&ctx, balance).unwrap().into_nodes();
        let mut app = App::new(source_display(input), tree, query_state());
        // Tree view is what turns balance rows into `RegisterScope::Subtree`.
        app.update(Message::Balance(BalanceMessage::ToggleTree));
        // The first parent row (a real ancestor) exercises `descendants_of`
        // with more than one underlying account.
        let scope = app
            .balance
            .rows
            .iter()
            .find(|r| r.has_children)
            .map(|r| r.scope)
            .expect("fixture has at least one parent account");
        let rows =
            super::super::fulfill::load_register(&mut ledger, &ctx, &template(), scope).unwrap();
        app.show_register(scope, rows);
        (ctx, app)
    }

    /// The default balance screen for every fixture ledger: a flat list of
    /// every account with its balance, the title bar, and the balance footer
    /// hints.
    #[rstest]
    fn render_balance(
        #[base_dir = "../testdata/report"]
        #[files("*.ledger")]
        input: PathBuf,
    ) {
        let arena = Bump::new();
        let (ctx, mut app) = balance_app(&arena, &input);
        golden(&golden_name(&input, "balance")).assert(&render(&mut app, &ctx));
    }

    /// The balance screen in tree (`DisplayMode::Tree`) mode for every fixture:
    /// the account hierarchy, indented with fold markers and subtree totals.
    #[rstest]
    fn render_balance_tree(
        #[base_dir = "../testdata/report"]
        #[files("*.ledger")]
        input: PathBuf,
    ) {
        let arena = Bump::new();
        let (ctx, mut app) = balance_app(&arena, &input);
        app.update(Message::Balance(BalanceMessage::ToggleTree)); // Flat -> Tree
        golden(&golden_name(&input, "balance-tree")).assert(&render(&mut app, &ctx));
    }

    /// The tree balance screen with every node collapsed (`x` fold-all): only
    /// the top-level accounts remain, each showing its whole subtree total.
    #[rstest]
    fn render_balance_tree_all_folded(
        #[base_dir = "../testdata/report"]
        #[files("*.ledger")]
        input: PathBuf,
    ) {
        let arena = Bump::new();
        let (ctx, mut app) = balance_app(&arena, &input);
        app.update(Message::Balance(BalanceMessage::ToggleTree)); // Flat -> Tree
        app.update(Message::Balance(BalanceMessage::ToggleFoldAll)); // collapse every node
        golden(&golden_name(&input, "balance-tree-all-folded")).assert(&render(&mut app, &ctx));
    }

    /// The register drill-down for every fixture ledger: dated rows with
    /// per-entry amount and running total for one account, plus the register
    /// footer hints.
    #[rstest]
    fn render_register(
        #[base_dir = "../testdata/report"]
        #[files("*.ledger")]
        input: PathBuf,
    ) {
        let arena = Bump::new();
        let (ctx, mut app) = register_app(&arena, &input);
        golden(&golden_name(&input, "register")).assert(&render(&mut app, &ctx));
    }

    /// The subtree register drill-down for every fixture ledger: dated rows
    /// aggregated across a parent account and all of its descendants.
    #[rstest]
    fn render_register_subtree(
        #[base_dir = "../testdata/report"]
        #[files("*.ledger")]
        input: PathBuf,
    ) {
        let arena = Bump::new();
        let (ctx, mut app) = register_subtree_app(&arena, &input);
        golden(&golden_name(&input, "register-subtree")).assert(&render(&mut app, &ctx));
    }

    /// The `many_commodities` balance: `Assets:Banks:Foo` holds 6 commodities
    /// and `Assets:Brokers:Bar` all 26 stock lots, against a 19-line body on an
    /// 80×24 terminal — more lines than one screen can hold.
    fn many_commodities_balance<'ctx>(arena: &'ctx Bump) -> (ReportContext<'ctx>, App<'ctx>) {
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../testdata/report/many_commodities.ledger");
        balance_app(arena, &input)
    }

    /// `↓` walks one commodity line; the account stays put until its lines run
    /// out. `J` skips the rest of them.
    #[test]
    fn item_step_moves_a_whole_account_at_a_time() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        render(&mut app, &ctx); // the first frame teaches the nav its viewport

        // Row 0 is the total; the first account is one item down.
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::NextItem)));
        assert_eq!(app.balance.nav.selected_item(), Some(1));
        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
        // Assets:Banks:Foo has six commodities; one line down is still inside it.
        assert_eq!(app.balance.nav.selected_item(), Some(1));

        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::NextItem)));
        assert_eq!(app.balance.nav.selected_item(), Some(2));
        // …and lands on the account's own first line, not partway into it.
        let nav = &app.balance.nav;
        assert_eq!(nav.lines().first_row(2), Some(nav.selected_row()));

        app.update(Message::Balance(BalanceMessage::Nav(NavCommand::PrevItem)));
        assert_eq!(app.balance.nav.selected_item(), Some(1));
    }

    /// An account taller than the body is reachable line by line, and the two
    /// things that used to be lost when it was — its name and the fact that
    /// there is more — are on screen.
    #[test]
    fn scrolling_into_a_tall_account_keeps_its_name_and_reports_the_cut() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        render(&mut app, &ctx);

        // Past the total and the bank account into the broker one, then down to
        // its 26th and last lot — far enough past the top of the body that its
        // label has scrolled off.
        for _ in 0..2 {
            app.update(Message::Balance(BalanceMessage::Nav(NavCommand::NextItem)));
        }
        for _ in 0..25 {
            app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
        }
        let out = render(&mut app, &ctx);
        assert_eq!(app.balance.nav.selected_item(), Some(2));
        assert!(
            out.contains("10 STOCKZ"),
            "the last lot should be reachable:\n{out}"
        );
        assert!(
            out.contains("Assets:Brokers:Bar"),
            "the account label should be redrawn on the top line:\n{out}"
        );
        assert!(
            out.contains("above"),
            "the top line should say how much is cut off above it:\n{out}"
        );
    }

    /// The sticky label is marked by colour alone — no prefix, so the text
    /// stays in the column it belongs to.
    #[test]
    fn sticky_account_label_is_drawn_in_its_own_color() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        render(&mut app, &ctx);
        // Past the total and the bank account, then deep into the broker one.
        for _ in 0..2 {
            app.update(Message::Balance(BalanceMessage::Nav(NavCommand::NextItem)));
        }
        for _ in 0..25 {
            app.update(Message::Balance(BalanceMessage::Nav(NavCommand::Down)));
        }

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &mut app, &ctx)).unwrap();
        let buf = terminal.backend().buffer().clone();

        // The body's first line: inside the border, below the header.
        let sticky: String = (1..79)
            .map(|x| buf[Position { x, y: 3 }].symbol())
            .collect();
        assert!(
            sticky.trim_end().ends_with("Assets:Brokers:Bar")
                || sticky.trim_start().starts_with("Assets:Brokers:Bar"),
            "expected the cut account's label on the top body line: {sticky:?}"
        );
        assert_eq!(buf[Position { x: 1, y: 3 }].fg, STICKY_COLOR);
    }

    /// A body with no room for a row after its header draws nothing at all, so
    /// it says why rather than showing an empty box.
    #[test]
    fn a_terminal_too_short_for_a_body_says_so() {
        let arena = Bump::new();
        let (ctx, mut app) = many_commodities_balance(&arena);
        // 1 title + 1 footer leaves 3 lines: two borders and the header, with
        // no room for a row.
        let out = render_sized(&mut app, &ctx, 40, 5);
        assert!(out.contains("terminal too short"), "{out}");
    }

    /// Renders `app` into a `width`×`height` headless terminal, plain text.
    fn render_sized<'ctx>(
        app: &mut App<'ctx>,
        ctx: &ReportContext<'ctx>,
        width: u16,
        height: u16,
    ) -> String {
        let backend = TestBackend::new(width, height);
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

    /// A register view backed by `n` real entries against one account, plus the
    /// context that keeps its arena references alive.
    fn register_app_with_rows<'ctx>(
        arena: &'ctx Bump,
        n: usize,
    ) -> (ReportContext<'ctx>, App<'ctx>) {
        use std::path::PathBuf;

        use maplit::hashmap;
        use okane_core::report::query::DateRange;
        use okane_core::{load, report};

        // Same date for every entry (distinct payees are what we assert on), so
        // we don't have to worry about month/day arithmetic.
        let mut content = String::new();
        for i in 0..n {
            content.push_str(&format!(
                "2024/01/01 Payee{i:02}\n    Assets:Cash    1 USD\n    Equity\n\n"
            ));
        }
        let map = hashmap! {
            PathBuf::from("test.ledger") => content.into_bytes(),
        };
        let loader = load::Loader::new(
            PathBuf::from("test.ledger"),
            load::FakeFileSystem::from(map),
        );
        let mut ctx = ReportContext::new(arena);
        let mut ledger =
            report::process(&mut ctx, loader, &report::ProcessOptions::default()).unwrap();
        let account = ctx.account("Assets:Cash").unwrap();
        let template = RegisterQueryTemplate {
            conversion: None,
            date_range: DateRange::default(),
        };
        let scope = super::super::register::RegisterScope::Single(account);
        let rows =
            super::super::fulfill::load_register(&mut ledger, &ctx, &template, scope).unwrap();
        assert_eq!(rows.len(), n);
        let mut app = App::new("test.ledger".to_owned(), Vec::new(), query_state());
        app.show_register(scope, rows);
        (ctx, app)
    }

    /// A register far longer than the viewport renders only a window: it opens
    /// on the last entry (bottom pinned) and scrolls to the top on `g`, showing
    /// entries at the far end but not the other.
    #[test]
    fn register_renders_only_the_visible_window() {
        let arena = Bump::new();
        let (ctx, mut app) = register_app_with_rows(&arena, 50);

        // Opens pinned to the last entry.
        let bottom = render_sized(&mut app, &ctx, 80, 10);
        assert!(bottom.contains("Payee49"), "last entry should be visible");
        assert!(
            !bottom.contains("Payee00"),
            "first entry should be scrolled off:\n{bottom}"
        );

        // Jump to the top: now the first entries show and the last are gone.
        app.update(Message::Register(RegisterMessage::Nav(NavCommand::First)));
        let top = render_sized(&mut app, &ctx, 80, 10);
        assert!(top.contains("Payee00"), "first entry should be visible");
        assert!(
            !top.contains("Payee49"),
            "last entry should be scrolled off:\n{top}"
        );
    }
}
