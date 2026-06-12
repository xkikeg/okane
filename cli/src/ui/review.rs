//! Interactive review session for `okane import --interactive`.
//!
//! Self-contained sibling of the balance TUI, following the same Elm-style
//! architecture: [`app::ReviewApp`] holds all state and exposes
//! [`app::ReviewApp::update`] for transitions; [`event`] translates
//! `KeyEvent`s to [`app::Message`]s, runs the loop, and fulfills
//! [`app::Command`]s against the mutable [`crate::import::LoadedImport`];
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

use crate::import::LoadedImport;

/// Runs the review TUI session with the prepared [`ReviewApp`].
///
/// Sets up the terminal (raw mode, alternate screen), runs the event loop,
/// and tears down the terminal on exit (normal or error). `loaded` is
/// borrowed mutably so user decisions can be applied to the transactions;
/// the caller writes them out afterwards if the outcome is
/// [`SessionOutcome::Write`].
pub fn run_review(
    app: &mut ReviewApp,
    loaded: &mut LoadedImport,
) -> anyhow::Result<SessionOutcome> {
    let mut terminal = ratatui::init();
    let result = event::run(&mut terminal, app, loaded);
    ratatui::restore();
    result
}
