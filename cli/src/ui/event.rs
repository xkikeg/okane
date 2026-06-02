//! Event loop and keyboard handling for the TUI.
//!
//! Architecture follows The Elm Architecture (see
//! <https://ratatui.rs/concepts/application-patterns/the-elm-architecture/>):
//! key events are translated into [`Message`]s by [`key_to_message`] (a pure
//! function that consults the current screen/overlay), [`App::update`] applies
//! them, and any returned [`Command`] is fulfilled here — the single place
//! that owns the mutable resources (the `Ledger`) the pure state machine
//! cannot touch.

use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lender::FallibleLender;
use okane_core::report::{Account, ReportContext};
use okane_core::report::query::{AccountFilter, Ledger, RegisterQuery, Sort};
use ratatui::DefaultTerminal;

use super::app::{
    App, Command, Message, Overlay, RegisterQueryTemplate, RegisterRow, Screen,
};
use super::render;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

/// Runs the event loop until the user quits.
pub fn run<'ctx>(
    terminal: &mut DefaultTerminal,
    app: &mut App<'ctx>,
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| render::draw(frame, app, ctx))?;
        if event::poll(POLL_TIMEOUT)?
            && let Event::Key(key) = event::read()?
            && let Some(msg) = key_to_message(app, key)
            && let Some(cmd) = app.update(msg)
        {
            fulfill(app, ledger, ctx, cmd)?;
        }
    }
    Ok(())
}

/// Pure: translate a raw `KeyEvent` into a [`Message`] given the current
/// screen and overlay. Returns `None` when the key is unmapped.
pub fn key_to_message(app: &App<'_>, key: KeyEvent) -> Option<Message> {
    // crossterm on Windows emits both Press and Release; act on Press
    // (and `Repeat`, treated like Press) to avoid double handling.
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // Ctrl-C always quits — even through an open overlay.
    if ctrl && matches!(key.code, KeyCode::Char('c')) {
        return Some(Message::QuitImmediate);
    }

    if app.overlay == Some(Overlay::QuitConfirm) {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Some(Message::ConfirmQuit)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                Some(Message::DismissOverlay)
            }
            _ => None,
        };
    }

    // Common navigation keys, regardless of screen.
    let nav = match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Message::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::MoveDown),
        KeyCode::PageUp => Some(Message::PageUp),
        KeyCode::Char('b') if ctrl => Some(Message::PageUp),
        KeyCode::PageDown => Some(Message::PageDown),
        KeyCode::Char('f') if ctrl => Some(Message::PageDown),
        KeyCode::Home | KeyCode::Char('g') => Some(Message::SelectFirst),
        KeyCode::End | KeyCode::Char('G') => Some(Message::SelectLast),
        _ => None,
    };
    if nav.is_some() {
        return nav;
    }

    // Screen-specific keys.
    match (&app.screen, key.code) {
        (Screen::Balance, KeyCode::Enter) => Some(Message::OpenRegister),
        (Screen::Balance, KeyCode::Char('q') | KeyCode::Esc) => Some(Message::RequestQuit),
        (Screen::Register(_), KeyCode::Char('q') | KeyCode::Esc) => {
            Some(Message::LeaveRegister)
        }
        _ => None,
    }
}

/// Executes a [`Command`] returned from [`App::update`]. The only impure
/// step in the loop — it owns the `&mut Ledger` the pure state machine
/// cannot touch.
fn fulfill<'ctx>(
    app: &mut App<'ctx>,
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
    cmd: Command<'ctx>,
) -> anyhow::Result<()> {
    match cmd {
        Command::LoadRegister { account } => {
            let rows = load_register(ledger, ctx, &app.register_template, account)
                .with_context(|| format!("failed to load register for {}", account.as_str()))?;
            app.show_register(account, rows);
            Ok(())
        }
    }
}

/// Collects the register rows for `account` into owned [`RegisterRow`]s so
/// they can be displayed without keeping the `FallibleLender` alive.
fn load_register<'ctx>(
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
    template: &RegisterQueryTemplate<'ctx>,
    account: Account<'ctx>,
) -> anyhow::Result<Vec<RegisterRow<'ctx>>> {
    let query = RegisterQuery {
        account: AccountFilter::single(account),
        date_range: template.date_range,
        conversion: template.conversion,
        sort: Sort::Date,
    };
    let mut entries = ledger.register_entries(ctx, &query)?;
    let mut rows = Vec::new();
    while let Some(entry) = entries.next()? {
        rows.push(RegisterRow {
            date: entry.date,
            payee: entry.payee.to_owned(),
            amount: entry.amount.clone(),
            total: entry.total.clone(),
        });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use bumpalo::Bump;
    use okane_core::{load, report};

    use super::*;
    use crate::ui::app::{RegisterView, TableNav};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn app<'ctx>() -> App<'ctx> {
        App::new(
            "test".to_owned(),
            Vec::new(),
            RegisterQueryTemplate {
                conversion: None,
                date_range: Default::default(),
            },
        )
    }

    /// Process a trivial ledger and return a resolved account.
    fn make_account<'ctx>(
        arena: &'ctx Bump,
        account_name: &str,
    ) -> (report::ReportContext<'ctx>, Account<'ctx>) {
        let content = format!("2024/01/01 Init\n    {account_name}    1 USD\n    Equity\n");
        let mut map = HashMap::new();
        map.insert(PathBuf::from("test.ledger"), content.into_bytes());
        let loader = load::Loader::new(
            PathBuf::from("test.ledger"),
            load::FakeFileSystem::from(map),
        );
        let mut ctx = report::ReportContext::new(arena);
        let _ =
            report::process(&mut ctx, loader, &report::ProcessOptions::default()).unwrap();
        let account = ctx.account(account_name).unwrap();
        (ctx, account)
    }

    #[test]
    fn balance_arrow_keys_map_to_nav() {
        let app = app();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Down)),
            Some(Message::MoveDown)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('k'))),
            Some(Message::MoveUp)
        );
    }

    #[test]
    fn balance_enter_opens_register() {
        let app = app();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::OpenRegister)
        );
    }

    #[test]
    fn balance_q_requests_quit_confirmation() {
        let app = app();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::RequestQuit)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::RequestQuit)
        );
    }

    #[test]
    fn register_q_leaves_register() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::LeaveRegister)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::LeaveRegister)
        );
    }

    #[test]
    fn register_enter_is_unmapped() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        assert_eq!(key_to_message(&app, key(KeyCode::Enter)), None);
    }

    #[test]
    fn ctrl_c_always_quits() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::QuitImmediate)
        );
        app.overlay = Some(Overlay::QuitConfirm);
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::QuitImmediate)
        );
        app.overlay = None;
        app.screen = Screen::Register(RegisterView {
            account,
            rows: Vec::new(),
            nav: TableNav::new(0),
        });
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::QuitImmediate)
        );
    }

    #[test]
    fn overlay_y_confirms_n_dismisses() {
        let mut app = app();
        app.overlay = Some(Overlay::QuitConfirm);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('y'))),
            Some(Message::ConfirmQuit)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('Y'))),
            Some(Message::ConfirmQuit)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::ConfirmQuit)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('n'))),
            Some(Message::DismissOverlay)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::DismissOverlay)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::DismissOverlay)
        );
    }

    #[test]
    fn key_release_is_ignored() {
        let app = app();
        let release =
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(key_to_message(&app, release), None);
    }
}
