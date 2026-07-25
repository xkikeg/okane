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
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use lender::FallibleLender;
use okane_core::report::ReportContext;
use okane_core::report::query::{AccountFilter, Ledger, RegisterQuery, Sort};
use ratatui::DefaultTerminal;

use crate::ui::keys::is_ctrl;
use crate::ui::table::key_to_nav;

use super::app::{App, Command, Message};
use super::overlay::Overlay;
use super::register::{RegisterQueryTemplate, RegisterRow, RegisterScope, RegisterView, Screen};
use super::render;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

/// Why the event loop returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RunOutcome {
    /// The user quit.
    Quit,
    /// The user asked for a reload; the session loop in [`super::run_ui`]
    /// fulfills it by tearing this session down and rebuilding from scratch.
    Reload,
}

/// Runs the event loop until the user quits or asks for a reload.
pub(super) fn run<'ctx>(
    terminal: &mut DefaultTerminal,
    app: &mut App<'ctx>,
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> anyhow::Result<RunOutcome> {
    while !app.should_quit {
        terminal.draw(|frame| render::draw(frame, app, ctx))?;
        if event::poll(POLL_TIMEOUT)?
            && let Event::Key(key) = event::read()?
            && let Some(msg) = key_to_message(app, key)
            && let Some(cmd) = app.update(msg)
        {
            match cmd {
                Command::Reload => return Ok(RunOutcome::Reload),
                Command::LoadRegister { scope } => {
                    let rows = load_register(ledger, ctx, &app.register_template, scope)
                        .with_context(|| {
                            format!("failed to load register for {}", scope.display_name())
                        })?;
                    app.show_register(scope, rows);
                }
            }
        }
    }
    Ok(RunOutcome::Quit)
}

/// Pure: translate a raw `KeyEvent` into a [`Message`] given the current
/// screen and overlay. Returns `None` when the key is unmapped.
fn key_to_message(app: &App<'_>, key: KeyEvent) -> Option<Message> {
    // crossterm on Windows emits both Press and Release; act on Press
    // (and `Repeat`, treated like Press) to avoid double handling.
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    let ctrl = is_ctrl(key.modifiers);

    // Ctrl-C always quits — even through an open overlay.
    if ctrl && matches!(key.code, KeyCode::Char('c')) {
        return Some(Message::QuitImmediate);
    }

    match &app.overlay {
        Some(Overlay::QuitConfirm) => {
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
        // Scrolling mirrors the table navigation keys so the modal needs no new
        // muscle memory. `r`/`F5` retry the load without a dismiss step; `q`
        // goes straight to the quit prompt.
        Some(Overlay::Error(_)) => {
            if let Some(cmd) = key_to_nav(key) {
                return Some(Message::OverlayScroll(cmd.into()));
            }
            return match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Message::DismissOverlay),
                KeyCode::Char('q') => Some(Message::RequestQuit),
                KeyCode::Char('r') | KeyCode::F(5) => Some(Message::Reload),
                _ => None,
            };
        }
        None => {}
    }

    // Each screen owns its own keys; the focused component maps them to its
    // message type. Keys it doesn't consume fall through to the global keys
    // below (quit, reload), which is how e.g. `q` on balance reaches the quit
    // prompt while `q` on register leaves the screen.
    let routed = match &app.screen {
        Screen::Balance => app.balance.key_to_message(key).map(Message::Balance),
        Screen::Register(_) => RegisterView::key_to_message(key).map(Message::Register),
    };
    if let Some(msg) = routed {
        return Some(msg);
    }

    // Global keys, for whatever the focused component left unmapped.
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::RequestQuit),
        KeyCode::Char('r') | KeyCode::F(5) => Some(Message::Reload),
        _ => None,
    }
}

/// Collects the register rows for `account` into owned [`RegisterRow`]s so
/// they can be displayed without keeping the `FallibleLender` alive.
pub fn load_register<'ctx>(
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
    template: &RegisterQueryTemplate<'ctx>,
    scope: RegisterScope<'ctx>,
) -> anyhow::Result<Vec<RegisterRow<'ctx>>> {
    let account = match scope {
        RegisterScope::Single(account) => AccountFilter::single(account),
        RegisterScope::Subtree(aggregate) => AccountFilter::descendants_of(ctx, aggregate),
    };
    let query = RegisterQuery {
        account,
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
    use super::*;

    use bumpalo::Bump;
    use crossterm::event::KeyModifiers;

    use okane_core::report::Account;

    use crate::ui::table::{NavCommand, TableNav};

    use super::super::balance::BalanceMessage;
    use super::super::overlay::{ErrorPopup, ScrollDelta};
    use super::super::register::{RegisterMessage, RegisterView};
    use super::super::search::{
        Search, SearchDirection, SearchIntent, SearchMatch, SearchMode, SearchPhase,
    };
    use super::super::testing::{make_account, template};

    /// A single-account register screen for `account`, empty rows.
    fn register_screen<'ctx>(account: Account<'ctx>) -> Screen<'ctx> {
        Screen::Register(RegisterView {
            scope: RegisterScope::Single(account),
            rows: Vec::new(),
            nav: TableNav::new(0),
            col_widths: None,
        })
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn app<'ctx>() -> App<'ctx> {
        App::new("test".to_owned(), Vec::new(), template())
    }

    #[test]
    fn balance_keys_route_to_the_balance_component() {
        let app = app();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Down)),
            Some(Message::Balance(BalanceMessage::Nav(NavCommand::Down)))
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('k'))),
            Some(Message::Balance(BalanceMessage::Nav(NavCommand::Up)))
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::Balance(BalanceMessage::OpenRegister))
        );
    }

    #[test]
    fn register_keys_route_to_the_register_component() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        app.screen = register_screen(account);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Down)),
            Some(Message::Register(RegisterMessage::Nav(NavCommand::Down)))
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
    fn register_q_routes_to_leave() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        app.screen = register_screen(account);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::Register(RegisterMessage::Leave))
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::Register(RegisterMessage::Leave))
        );
    }

    #[test]
    fn register_enter_is_unmapped() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        app.screen = register_screen(account);
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
        app.screen = register_screen(account);
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

    fn app_with_error_modal<'ctx>() -> App<'ctx> {
        let mut app = app();
        app.overlay = Some(Overlay::Error(ErrorPopup::new(
            "failed to load test.ledger".to_owned(),
            vec!["boom".to_owned()],
        )));
        app
    }

    #[test]
    fn error_overlay_scroll_keys() {
        let app = app_with_error_modal();
        for (k, delta) in [
            (key(KeyCode::Char('j')), ScrollDelta::Lines(1)),
            (key(KeyCode::Down), ScrollDelta::Lines(1)),
            (ctrl_key('n'), ScrollDelta::Lines(1)),
            (key(KeyCode::Char('k')), ScrollDelta::Lines(-1)),
            (key(KeyCode::Up), ScrollDelta::Lines(-1)),
            (ctrl_key('p'), ScrollDelta::Lines(-1)),
            (key(KeyCode::PageDown), ScrollDelta::Pages(1)),
            (ctrl_key('f'), ScrollDelta::Pages(1)),
            (key(KeyCode::PageUp), ScrollDelta::Pages(-1)),
            (ctrl_key('b'), ScrollDelta::Pages(-1)),
            (key(KeyCode::Char('g')), ScrollDelta::Top),
            (key(KeyCode::Home), ScrollDelta::Top),
            (key(KeyCode::Char('G')), ScrollDelta::Bottom),
            (key(KeyCode::End), ScrollDelta::Bottom),
        ] {
            assert_eq!(
                key_to_message(&app, k),
                Some(Message::OverlayScroll(delta)),
                "{k:?}"
            );
        }
    }

    #[test]
    fn error_overlay_dismiss_and_quit_keys() {
        let app = app_with_error_modal();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::DismissOverlay)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::DismissOverlay)
        );
        // Unlike the quit prompt, `q` here means quit, matching the normal
        // balance screen.
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::RequestQuit)
        );
    }

    /// Retrying straight from the modal — contrast with
    /// [`r_during_quit_overlay_is_unmapped`].
    #[test]
    fn error_overlay_r_and_f5_reload() {
        let app = app_with_error_modal();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('r'))),
            Some(Message::Reload)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::F(5))),
            Some(Message::Reload)
        );
    }

    #[test]
    fn error_overlay_swallows_other_keys() {
        let app = app_with_error_modal();
        assert_eq!(key_to_message(&app, key(KeyCode::Char('/'))), None);
        assert_eq!(key_to_message(&app, key(KeyCode::Char('y'))), None);
        assert_eq!(key_to_message(&app, ctrl_key('s')), None);
    }

    #[test]
    fn ctrl_c_quits_through_the_error_modal() {
        let app = app_with_error_modal();
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::QuitImmediate)
        );
    }

    fn fixed_search() -> Search {
        Search {
            intent: SearchIntent {
                mode: SearchMode::Modal(SearchPhase::Fixed),
                dir: SearchDirection::Forward,
                input: "a".to_owned(),
                no_previous: false,
                origin: 0,
            },
            matches: Some(Ok(SearchMatch::from(vec![0]))),
        }
    }

    #[test]
    fn key_release_is_ignored() {
        let app = app();
        let release =
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(key_to_message(&app, release), None);
    }

    #[test]
    fn plain_r_and_f5_reload_on_both_screens() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:A");
        let mut app = app();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('r'))),
            Some(Message::Reload)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::F(5))),
            Some(Message::Reload)
        );
        app.screen = register_screen(account);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('r'))),
            Some(Message::Reload)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::F(5))),
            Some(Message::Reload)
        );
    }

    #[test]
    fn r_during_fixed_search_reloads() {
        let mut app = app();
        app.balance.search = Some(fixed_search());
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('r'))),
            Some(Message::Reload)
        );
    }

    #[test]
    fn r_during_quit_overlay_is_unmapped() {
        let mut app = app();
        app.overlay = Some(Overlay::QuitConfirm);
        assert_eq!(key_to_message(&app, key(KeyCode::Char('r'))), None);
        assert_eq!(key_to_message(&app, key(KeyCode::F(5))), None);
    }
}
