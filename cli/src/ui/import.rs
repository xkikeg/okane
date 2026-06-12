//! Interactive review session for `okane import --interactive`.
//!
//! Follow the same Elm-style architecture as the report TUI:
//! [`app::ReviewApp`] holds all state and exposes
//! [`app::ReviewApp::update`] for transitions; [`event`] translates
//! `KeyEvent`s to [`app::Message`]s, runs the loop, and fulfills
//! [`app::Command`]s against the mutable transaction list;
//! [`render`] is a pure view over the app state.
//!
//! The session walks the imported transactions: rule-matched ones marked
//! `pending: true` are confirmed with `a`, unknown ones get a destination
//! account assigned through an autocomplete prompt (`e`/Enter). `w` writes
//! and quits, `q` aborts without writing.

pub mod app;
mod event;
mod render;

pub use app::{ReviewApp, ReviewItem, SessionOutcome};

use crate::import::{ImportHeader, single_entry};

/// Runs the review TUI session with the prepared [`ReviewApp`].
///
/// Sets up the terminal (raw mode, alternate screen), runs the event loop,
/// and tears down the terminal on exit (normal or error). `txns` is borrowed
/// mutably so user decisions can be applied; the caller writes them out
/// afterwards if the outcome is [`SessionOutcome::Write`].
pub fn run_review(
    app: &mut ReviewApp,
    header: &ImportHeader,
    txns: &mut [single_entry::Txn],
) -> anyhow::Result<SessionOutcome> {
    let mut terminal = ratatui::init();
    let result = event::run(&mut terminal, app, header, txns);
    ratatui::restore();
    result
}
