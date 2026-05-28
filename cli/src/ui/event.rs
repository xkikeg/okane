//! Event loop and keyboard handling for the TUI.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use okane_core::report::ReportContext;
use ratatui::DefaultTerminal;

use super::app::{App, TableNav};
use super::render;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

/// Runs the event loop until the user quits.
pub fn run<'ctx>(
    terminal: &mut DefaultTerminal,
    app: &mut App<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> io::Result<()> {
    while !app.nav.should_quit {
        terminal.draw(|frame| render::draw(frame, app, ctx))?;
        if event::poll(POLL_TIMEOUT)?
            && let Event::Key(key) = event::read()?
        {
            handle_key_event(&mut app.nav, key);
        }
    }
    Ok(())
}

/// Applies a key event to the navigation state.
///
/// Pure: only mutates `nav`, no IO. Used by the event loop and by unit tests
/// to drive synthetic input without a real terminal or a `'ctx`-bound `App`.
pub fn handle_key_event(nav: &mut TableNav, key: KeyEvent) {
    // crossterm on Windows emits both Press and Release events; act on Press
    // (and `Repeat`, treated like Press) to avoid double handling.
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => nav.quit(),
        KeyCode::Char('c') if ctrl => nav.quit(),
        KeyCode::Up | KeyCode::Char('k') => nav.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => nav.move_selection(1),
        KeyCode::PageUp => {
            let delta = -(nav.page_size() as isize);
            nav.move_selection(delta);
        }
        KeyCode::Char('b') if ctrl => {
            let delta = -(nav.page_size() as isize);
            nav.move_selection(delta);
        }
        KeyCode::PageDown => {
            let delta = nav.page_size() as isize;
            nav.move_selection(delta);
        }
        KeyCode::Char('f') if ctrl => {
            let delta = nav.page_size() as isize;
            nav.move_selection(delta);
        }
        KeyCode::Home | KeyCode::Char('g') => nav.select_first(),
        KeyCode::End | KeyCode::Char('G') => nav.select_last(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn arrow_keys_move_selection() {
        let mut nav = TableNav::new(5);
        handle_key_event(&mut nav, key(KeyCode::Down));
        assert_eq!(nav.table_state.selected(), Some(1));
        handle_key_event(&mut nav, key(KeyCode::Down));
        assert_eq!(nav.table_state.selected(), Some(2));
        handle_key_event(&mut nav, key(KeyCode::Up));
        assert_eq!(nav.table_state.selected(), Some(1));
    }

    #[test]
    fn vim_keys_move_selection() {
        let mut nav = TableNav::new(5);
        handle_key_event(&mut nav, key(KeyCode::Char('j')));
        assert_eq!(nav.table_state.selected(), Some(1));
        handle_key_event(&mut nav, key(KeyCode::Char('k')));
        assert_eq!(nav.table_state.selected(), Some(0));
    }

    #[test]
    fn page_keys_use_viewport_height() {
        let mut nav = TableNav::new(50);
        nav.viewport_height = 10;
        handle_key_event(&mut nav, key(KeyCode::PageDown));
        assert_eq!(nav.table_state.selected(), Some(10));
        handle_key_event(&mut nav, ctrl_key('f'));
        assert_eq!(nav.table_state.selected(), Some(20));
        handle_key_event(&mut nav, key(KeyCode::PageUp));
        assert_eq!(nav.table_state.selected(), Some(10));
        handle_key_event(&mut nav, ctrl_key('b'));
        assert_eq!(nav.table_state.selected(), Some(0));
    }

    #[test]
    fn home_and_end_jump_to_bounds() {
        let mut nav = TableNav::new(10);
        handle_key_event(&mut nav, key(KeyCode::Char('G')));
        assert_eq!(nav.table_state.selected(), Some(9));
        handle_key_event(&mut nav, key(KeyCode::Char('g')));
        assert_eq!(nav.table_state.selected(), Some(0));
        handle_key_event(&mut nav, key(KeyCode::End));
        assert_eq!(nav.table_state.selected(), Some(9));
        handle_key_event(&mut nav, key(KeyCode::Home));
        assert_eq!(nav.table_state.selected(), Some(0));
    }

    #[test]
    fn quit_keys_set_should_quit() {
        for code in [KeyCode::Char('q'), KeyCode::Esc] {
            let mut nav = TableNav::new(3);
            handle_key_event(&mut nav, key(code));
            assert!(nav.should_quit, "quit key {code:?} did not quit");
        }
        let mut nav = TableNav::new(3);
        handle_key_event(&mut nav, ctrl_key('c'));
        assert!(nav.should_quit, "Ctrl-C did not quit");
    }

    #[test]
    fn unmapped_key_is_noop() {
        let mut nav = TableNav::new(3);
        handle_key_event(&mut nav, key(KeyCode::Char('x')));
        assert_eq!(nav.table_state.selected(), Some(0));
        assert!(!nav.should_quit);
    }

    #[test]
    fn key_release_is_ignored() {
        let mut nav = TableNav::new(5);
        let release =
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Release);
        handle_key_event(&mut nav, release);
        assert_eq!(nav.table_state.selected(), Some(0));
    }
}
