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

use super::app::{
    App, Command, Drill, Message, Overlay, RegisterQueryTemplate, RegisterRow, Screen, ScrollDelta,
    SearchDirection, SearchMode, SearchPhase,
};
use super::render;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

fn scroll(delta: ScrollDelta) -> Option<Message> {
    Some(Message::OverlayScroll(delta))
}

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
                Command::LoadRegister { drill } => {
                    let title = drill.display_name().to_owned();
                    let rows = load_register(ledger, ctx, &app.register_template, drill)
                        .with_context(|| format!("failed to load register for {title}"))?;
                    app.show_register(drill, title, rows);
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
        // Scrolling mirrors the common navigation block below so the modal
        // needs no new muscle memory. `r`/`F5` retry the load without a
        // dismiss step; `q` goes straight to the quit prompt.
        Some(Overlay::Error(_)) => {
            return match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Message::DismissOverlay),
                KeyCode::Char('q') => Some(Message::RequestQuit),
                KeyCode::Up | KeyCode::Char('k') => scroll(ScrollDelta::Lines(-1)),
                KeyCode::Char('p') if ctrl => scroll(ScrollDelta::Lines(-1)),
                KeyCode::Down | KeyCode::Char('j') => scroll(ScrollDelta::Lines(1)),
                KeyCode::Char('n') if ctrl => scroll(ScrollDelta::Lines(1)),
                KeyCode::PageUp => scroll(ScrollDelta::Pages(-1)),
                KeyCode::Char('b') if ctrl => scroll(ScrollDelta::Pages(-1)),
                KeyCode::PageDown => scroll(ScrollDelta::Pages(1)),
                KeyCode::Char('f') if ctrl => scroll(ScrollDelta::Pages(1)),
                KeyCode::Home | KeyCode::Char('g') => scroll(ScrollDelta::Top),
                KeyCode::End | KeyCode::Char('G') => scroll(ScrollDelta::Bottom),
                KeyCode::Char('r') | KeyCode::F(5) => Some(Message::Reload),
                _ => None,
            };
        }
        None => {}
    }

    // Balance account-search capture. The editing phases (modal incremental,
    // interactive i-search) own every key; the modal fixed phase intercepts
    // only its own controls and lets the rest fall through so full navigation
    // (and Enter-to-register) keep working.
    if let Some(search) = &app.search {
        match search.intent.mode {
            SearchMode::Modal(SearchPhase::Incremental) => {
                return match key.code {
                    KeyCode::Esc => Some(Message::SearchCancel),
                    KeyCode::Enter => Some(Message::SearchSubmit),
                    KeyCode::Backspace => Some(Message::SearchPop),
                    // TODO: now we're pushing char also with modifier,
                    // which isn't good. probably let widget hold the text,
                    // and pass them entirely.
                    KeyCode::Char(c) if !ctrl => Some(Message::SearchPush(c)),
                    _ => None,
                };
            }
            SearchMode::Modal(SearchPhase::Fixed) => match key.code {
                KeyCode::Esc => return Some(Message::SearchClose),
                KeyCode::Char('n') => return Some(Message::SearchNext),
                KeyCode::Char('N') => return Some(Message::SearchPrev),
                _ => {} // fallback to normal UI
            },
            SearchMode::Interactive => {
                // Canonical i-search: editing is always live;
                // C-g/Esc aborts to the origin. RET drills into the
                // register and C-n/C-p move the selection — these three end
                // the search (keeping the cursor), behaving like normal view.
                match key.code {
                    KeyCode::Char('s') if ctrl => return Some(Message::SearchNext),
                    KeyCode::Char('r') if ctrl => return Some(Message::SearchPrev),
                    KeyCode::Char('g') if ctrl => return Some(Message::SearchCancel),
                    KeyCode::Esc => return Some(Message::SearchCancel),
                    KeyCode::Backspace => return Some(Message::SearchPop),
                    KeyCode::Char(c) if !ctrl => return Some(Message::SearchPush(c)),
                    _ => {} // fallback to normal UI
                };
            }
        }
    }

    // Common navigation keys, regardless of screen.
    let nav = match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Message::MoveUp),
        KeyCode::Char('p') if ctrl => Some(Message::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::MoveDown),
        KeyCode::Char('n') if ctrl => Some(Message::MoveDown),
        KeyCode::PageUp => Some(Message::PageUp),
        KeyCode::Char('b') if ctrl => Some(Message::PageUp),
        KeyCode::PageDown => Some(Message::PageDown),
        KeyCode::Char('f') if ctrl => Some(Message::PageDown),
        KeyCode::Home | KeyCode::Char('g') => Some(Message::SelectFirst),
        KeyCode::End | KeyCode::Char('G') => Some(Message::SelectLast),
        KeyCode::F(5) => Some(Message::Reload),
        _ => None,
    };
    if nav.is_some() {
        return nav;
    }

    // Screen-specific keys.
    match (&app.screen, key.code) {
        (Screen::Balance, KeyCode::Char('/')) => Some(Message::StartModalSearch),
        (Screen::Balance, KeyCode::Char('s')) if ctrl => {
            Some(Message::StartISearch(SearchDirection::Forward))
        }
        (Screen::Balance, KeyCode::Char('r')) if ctrl => {
            Some(Message::StartISearch(SearchDirection::Backward))
        }
        (Screen::Balance, KeyCode::Enter) => Some(Message::OpenRegister),
        (Screen::Balance, KeyCode::Char('t')) => Some(Message::ToggleTree),
        (Screen::Balance, KeyCode::Char(' ')) => Some(Message::ToggleFold),
        (Screen::Balance, KeyCode::Char('x')) => Some(Message::ToggleFoldAll),
        (Screen::Balance, KeyCode::Char('q') | KeyCode::Esc) => Some(Message::RequestQuit),
        (Screen::Balance | Screen::Register(_), KeyCode::Char('r')) => Some(Message::Reload),
        (Screen::Register(_), KeyCode::Char('q') | KeyCode::Esc) => Some(Message::LeaveRegister),
        _ => None,
    }
}

/// Collects the register rows for `account` into owned [`RegisterRow`]s so
/// they can be displayed without keeping the `FallibleLender` alive.
pub fn load_register<'ctx>(
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
    template: &RegisterQueryTemplate<'ctx>,
    drill: Drill<'ctx>,
) -> anyhow::Result<Vec<RegisterRow<'ctx>>> {
    let account = match drill {
        Drill::Single(account) => AccountFilter::single(account),
        Drill::Subtree(aggregate) => AccountFilter::descendants_of(ctx, aggregate),
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

    use std::collections::HashMap;
    use std::path::PathBuf;

    use bumpalo::Bump;
    use crossterm::event::KeyModifiers;
    use okane_core::{load, report};

    use okane_core::report::Account;

    use crate::ui::table::TableNav;

    use super::super::app::{ErrorPopup, RegisterView, Search, SearchIntent, SearchMatch};

    /// A single-account register screen for `account`, empty rows.
    fn register_screen<'ctx>(account: Account<'ctx>) -> Screen<'ctx> {
        Screen::Register(RegisterView {
            drill: Drill::Single(account),
            title: account.as_str().to_owned(),
            rows: Vec::new(),
            nav: TableNav::new(0),
        })
    }

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
        let _ = report::process(&mut ctx, loader, &report::ProcessOptions::default()).unwrap();
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
        app.screen = register_screen(account);
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

    fn interactive_search() -> Search {
        Search {
            intent: SearchIntent {
                mode: SearchMode::Interactive,
                dir: SearchDirection::Forward,
                input: "a".to_owned(),
                no_previous: false,
                origin: 0,
            },
            matches: Some(Ok(SearchMatch::from(vec![0]))),
        }
    }

    #[test]
    fn balance_slash_starts_search() {
        let app = app();
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('/'))),
            Some(Message::StartModalSearch)
        );
    }

    #[test]
    fn incremental_search_captures_editing_keys() {
        let mut app = app();
        app.update(Message::StartModalSearch);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('j'))),
            Some(Message::SearchPush('j'))
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Backspace)),
            Some(Message::SearchPop)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::SearchSubmit)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::SearchCancel)
        );
    }

    #[test]
    fn fixed_search_intercepts_only_its_controls() {
        let mut app = app();
        app.search = Some(fixed_search());
        // Own controls.
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('n'))),
            Some(Message::SearchNext)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('N'))),
            Some(Message::SearchPrev)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::SearchClose)
        );
        // Everything else falls through to normal navigation / drill-in.
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('j'))),
            Some(Message::MoveDown)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::OpenRegister)
        );
    }

    #[test]
    fn balance_ctrl_s_and_r_start_isearch() {
        let app = app();
        assert_eq!(
            key_to_message(&app, ctrl_key('s')),
            Some(Message::StartISearch(SearchDirection::Forward))
        );
        assert_eq!(
            key_to_message(&app, ctrl_key('r')),
            Some(Message::StartISearch(SearchDirection::Backward))
        );
    }

    #[test]
    fn interactive_search_captures_keys() {
        let mut app = app();
        app.search = Some(interactive_search());
        // Plain characters refine the pattern.
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('j'))),
            Some(Message::SearchPush('j'))
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Backspace)),
            Some(Message::SearchPop)
        );
        // C-s / C-r repeat; C-g and Esc abort.
        assert_eq!(
            key_to_message(&app, ctrl_key('s')),
            Some(Message::SearchNext)
        );
        assert_eq!(
            key_to_message(&app, ctrl_key('r')),
            Some(Message::SearchPrev)
        );
        assert_eq!(
            key_to_message(&app, ctrl_key('g')),
            Some(Message::SearchCancel)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::SearchCancel)
        );
        // RET drills into the register; C-n/C-p move — all end the search.
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::OpenRegister)
        );
        assert_eq!(key_to_message(&app, ctrl_key('n')), Some(Message::MoveDown));
        assert_eq!(key_to_message(&app, ctrl_key('p')), Some(Message::MoveUp));
    }

    #[test]
    fn ctrl_n_and_p_navigate_like_j_k() {
        let app = app();
        assert_eq!(key_to_message(&app, ctrl_key('n')), Some(Message::MoveDown));
        assert_eq!(key_to_message(&app, ctrl_key('p')), Some(Message::MoveUp));
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
    fn r_during_search_editing_is_captured_as_input() {
        // Modal incremental: every character belongs to the pattern.
        let mut modal_app = app();
        modal_app.update(Message::StartModalSearch);
        assert_eq!(
            key_to_message(&modal_app, key(KeyCode::Char('r'))),
            Some(Message::SearchPush('r'))
        );
        // Interactive i-search: same.
        let mut isearch_app = app();
        isearch_app.search = Some(interactive_search());
        assert_eq!(
            key_to_message(&isearch_app, key(KeyCode::Char('r'))),
            Some(Message::SearchPush('r'))
        );
    }

    #[test]
    fn r_during_fixed_search_reloads() {
        let mut app = app();
        app.search = Some(fixed_search());
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
