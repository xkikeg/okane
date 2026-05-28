//! Terminal UI for okane — entry point and shared types.
//!
//! See [`run_ui`] for the public entry point. The first iteration renders the
//! same content as `okane balance` as a single scrollable table.

mod app;
mod event;
mod render;

pub use app::{App, BalanceRow};

use std::io;

use okane_core::report::ReportContext;

/// Runs the TUI session with the prepared [`App`].
///
/// Sets up the terminal (raw mode, alternate screen), installs a panic hook
/// that restores the terminal before propagating, runs the event loop, and
/// tears down the terminal on exit (normal or error).
///
/// `ctx` is borrowed for the lifetime of the session so amount values can be
/// formatted lazily under the same `ReportContext` that produced them.
pub fn run_ui<'ctx>(mut app: App<'ctx>, ctx: &ReportContext<'ctx>) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event::run(&mut terminal, &mut app, ctx);
    ratatui::restore();
    result
}
