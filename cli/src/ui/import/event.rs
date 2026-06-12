//! Event loop and keyboard handling for the review TUI.
//!
//! Key events are translated into [`Message`]s by [`key_to_message`] (a pure
//! function that consults the active prompt/overlay), [`ReviewApp::update`]
//! applies them, and any returned [`Command`] is fulfilled here — the single
//! place that owns the mutable transaction list the pure state machine
//! cannot touch.

use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use okane_core::syntax::ClearState;
use ratatui::DefaultTerminal;

use crate::import::{ImportHeader, single_entry};

use super::app::{Command, Message, ReviewApp, SessionOutcome};
use super::render;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

/// Runs the event loop until the session outcome is decided.
pub fn run(
    terminal: &mut DefaultTerminal,
    app: &mut ReviewApp,
    header: &ImportHeader,
    txns: &mut [single_entry::Txn],
) -> anyhow::Result<SessionOutcome> {
    loop {
        if let Some(outcome) = app.outcome {
            return Ok(outcome);
        }
        terminal.draw(|frame| render::draw(frame, app))?;
        if event::poll(POLL_TIMEOUT)?
            && let Event::Key(key) = event::read()?
            && let Some(msg) = key_to_message(app, key)
            && let Some(cmd) = app.update(msg)
        {
            fulfill(app, header, txns, cmd)?;
        }
    }
}

/// Pure: translate a raw `KeyEvent` into a [`Message`] given the active
/// prompt/overlay. Returns `None` when the key is unmapped.
pub fn key_to_message(app: &ReviewApp, key: KeyEvent) -> Option<Message> {
    // crossterm on Windows emits both Press and Release; act on Press
    // (and `Repeat`, treated like Press) to avoid double handling.
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    let ctrl = (key.modifiers & !KeyModifiers::SHIFT) == KeyModifiers::CONTROL;

    // Ctrl-C always aborts — even through an open overlay or prompt.
    if ctrl && matches!(key.code, KeyCode::Char('c')) {
        return Some(Message::AbortImmediate);
    }

    if app.overlay.is_some() {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Some(Message::ConfirmOverlay)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                Some(Message::DismissOverlay)
            }
            _ => None,
        };
    }

    // The prompt captures almost everything so account names can contain
    // letters that are otherwise bound (a/e/s/w/q...).
    if app.prompt.is_some() {
        return match key.code {
            KeyCode::Esc => Some(Message::PromptCancel),
            KeyCode::Char('g') if ctrl => Some(Message::PromptCancel),
            KeyCode::Enter => Some(Message::PromptSubmit),
            KeyCode::Backspace => Some(Message::PromptBackspace),
            KeyCode::Tab => Some(Message::PromptComplete),
            KeyCode::Up => Some(Message::PromptPrev),
            KeyCode::Char('p') if ctrl => Some(Message::PromptPrev),
            KeyCode::Down => Some(Message::PromptNext),
            KeyCode::Char('n') if ctrl => Some(Message::PromptNext),
            KeyCode::Char(c) if !ctrl => Some(Message::PromptInput(c)),
            _ => None,
        };
    }

    // Common navigation keys.
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
        _ => None,
    };
    if nav.is_some() {
        return nav;
    }

    match key.code {
        KeyCode::Char('a') => Some(Message::Accept),
        KeyCode::Char('e') | KeyCode::Enter => Some(Message::OpenPrompt),
        KeyCode::Char('s') => Some(Message::Skip),
        KeyCode::Char('w') => Some(Message::RequestWrite),
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::RequestAbort),
        _ => None,
    }
}

/// Executes a [`Command`] returned from [`ReviewApp::update`].
fn fulfill(
    app: &mut ReviewApp,
    header: &ImportHeader,
    txns: &mut [single_entry::Txn],
    cmd: Command,
) -> anyhow::Result<()> {
    match cmd {
        Command::AcceptPending { index } => {
            let txn: &mut _ = txns.get_mut(index).context("out-of-range access")?;
            txn.clear_state(ClearState::Uncleared);
            let preview = header
                .render_transaction(txn)
                .with_context(|| format!("failed to render transaction {}", index + 1))?;
            app.apply_decision(index, preview);
        }
        Command::SetAccount { index, account } => {
            let txn: &mut _ = txns.get_mut(index).context("out-of-range access")?;
            txn.dest_account(&account)
                .clear_state(ClearState::Uncleared);
            let preview = header
                .render_transaction(txn)
                .with_context(|| format!("failed to render transaction {}", index + 1))?;
            app.apply_decision(index, preview);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::import::single_entry::ReviewKind;
    use crate::ui::import::app::ReviewItem;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn app(kinds: &[ReviewKind]) -> ReviewApp {
        let items = kinds
            .iter()
            .map(|&kind| {
                ReviewItem::new(
                    kind,
                    "preview\n".to_string(),
                    NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                    "payee".to_string(),
                    "10 USD".to_string(),
                )
            })
            .collect();
        ReviewApp::new(
            "source.csv".to_string(),
            "out.ledger".to_string(),
            items,
            vec!["Assets:Bank".to_string()],
        )
    }

    #[test]
    fn queue_keys_map_to_actions() {
        let app = app(&[ReviewKind::Unknown]);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('a'))),
            Some(Message::Accept)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('e'))),
            Some(Message::OpenPrompt)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::OpenPrompt)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('s'))),
            Some(Message::Skip)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('w'))),
            Some(Message::RequestWrite)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::RequestAbort)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::RequestAbort)
        );
    }

    #[test]
    fn queue_nav_keys_map_to_nav() {
        let app = app(&[ReviewKind::Unknown]);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Down)),
            Some(Message::MoveDown)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('j'))),
            Some(Message::MoveDown)
        );
        assert_eq!(key_to_message(&app, ctrl_key('n')), Some(Message::MoveDown));
        assert_eq!(key_to_message(&app, ctrl_key('p')), Some(Message::MoveUp));
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('g'))),
            Some(Message::SelectFirst)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('G'))),
            Some(Message::SelectLast)
        );
    }

    #[test]
    fn prompt_captures_bound_letters_as_input() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        // 'q' must type into the prompt, not abort the session.
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('q'))),
            Some(Message::PromptInput('q'))
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('a'))),
            Some(Message::PromptInput('a'))
        );
    }

    #[test]
    fn prompt_control_keys() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::PromptSubmit)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::PromptCancel)
        );
        assert_eq!(
            key_to_message(&app, ctrl_key('g')),
            Some(Message::PromptCancel)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Tab)),
            Some(Message::PromptComplete)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Backspace)),
            Some(Message::PromptBackspace)
        );
        assert_eq!(
            key_to_message(&app, ctrl_key('n')),
            Some(Message::PromptNext)
        );
        assert_eq!(
            key_to_message(&app, ctrl_key('p')),
            Some(Message::PromptPrev)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Up)),
            Some(Message::PromptPrev)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Down)),
            Some(Message::PromptNext)
        );
    }

    #[test]
    fn overlay_y_confirms_n_dismisses() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::RequestAbort);
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('y'))),
            Some(Message::ConfirmOverlay)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Enter)),
            Some(Message::ConfirmOverlay)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Char('n'))),
            Some(Message::DismissOverlay)
        );
        assert_eq!(
            key_to_message(&app, key(KeyCode::Esc)),
            Some(Message::DismissOverlay)
        );
    }

    #[test]
    fn ctrl_c_aborts_through_prompt_and_overlay() {
        let mut app = app(&[ReviewKind::Unknown]);
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::AbortImmediate)
        );
        app.update(Message::OpenPrompt);
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::AbortImmediate)
        );
        app.update(Message::PromptCancel);
        app.update(Message::RequestAbort);
        assert_eq!(
            key_to_message(&app, ctrl_key('c')),
            Some(Message::AbortImmediate)
        );
    }

    #[test]
    fn key_release_is_ignored() {
        let app = app(&[ReviewKind::Unknown]);
        let release =
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(key_to_message(&app, release), None);
    }
}
