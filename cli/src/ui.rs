//! Terminal UI for okane — entry point and shared types.
//!
//! Architecture (Elm-style; see
//! <https://ratatui.rs/concepts/application-patterns/the-elm-architecture/>):
//! - [`App`] holds all state and exposes [`App::update`] for transitions.
//! - [`event`] translates `KeyEvent`s to [`app::Message`]s and runs the loop.
//! - [`render`] is a pure view over `App`.
//!
//! See [`run_ui`] for the public entry point. The session presents a balance
//! table; pressing Enter drills into a register view for the selected
//! account, and q/Esc returns. A quit confirmation popup guards exits from
//! the balance screen; Ctrl-C bypasses it.

mod app;
mod event;
mod render;
mod import;

pub use app::{App, BalanceRow, RegisterQueryTemplate};
pub use import::{ReviewApp, ReviewItem, SessionOutcome, run_review};

use okane_core::report::ReportContext;
use okane_core::report::query::Ledger;

/// Runs the TUI session with the prepared [`App`].
///
/// Sets up the terminal (raw mode, alternate screen), installs a panic hook
/// that restores the terminal before propagating, runs the event loop, and
/// tears down the terminal on exit (normal or error).
///
/// `ledger` is borrowed mutably so the register view can be computed on
/// demand. `ctx` is borrowed for the lifetime of the session so amount
/// values can be formatted lazily under the same `ReportContext` that
/// produced them.
pub fn run_ui<'ctx>(
    mut app: App<'ctx>,
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = event::run(&mut terminal, &mut app, ledger, ctx);
    ratatui::restore();
    result
}
