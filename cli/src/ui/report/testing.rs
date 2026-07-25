//! Shared unit-test helpers for the report TUI modules.
//!
//! These build a report session from an in-memory ledger (via
//! [`load::FakeFileSystem`]), so `app`, `event`, `render`, and the `report`
//! session-loop tests can share one copy instead of each rolling its own.

use std::path::PathBuf;

use bumpalo::Bump;
use maplit::hashmap;
use okane_core::load;
use okane_core::report::query::{DateRange, Ledger};
use okane_core::report::{self, Account, ReportContext};

use super::register::RegisterQueryTemplate;

/// A register query template with no conversion over the full date range.
pub(super) fn template<'ctx>() -> RegisterQueryTemplate<'ctx> {
    RegisterQueryTemplate {
        conversion: None,
        date_range: DateRange::default(),
    }
}

/// A [`load::Loader`] over a single in-memory `test.ledger` holding `content`.
pub(super) fn fake_loader(content: &str) -> load::Loader<load::FakeFileSystem> {
    let map = hashmap! {
        PathBuf::from("test.ledger") => content.as_bytes().to_vec(),
    };
    load::Loader::new(
        PathBuf::from("test.ledger"),
        load::FakeFileSystem::from(map),
    )
}

/// Loads and processes `content` into a fresh context, returning both the
/// context and the resulting ledger.
pub(super) fn process<'ctx>(
    arena: &'ctx Bump,
    content: &str,
) -> (ReportContext<'ctx>, Ledger<'ctx>) {
    let mut ctx = ReportContext::new(arena);
    let ledger = report::process(
        &mut ctx,
        fake_loader(content),
        &report::ProcessOptions::default(),
    )
    .expect("test ledger should process");
    (ctx, ledger)
}

/// Processes a trivial ledger posting to `account_name` and returns the
/// context plus that resolved account.
pub(super) fn make_account<'ctx>(
    arena: &'ctx Bump,
    account_name: &str,
) -> (ReportContext<'ctx>, Account<'ctx>) {
    let content = format!("2024/01/01 Init\n    {account_name}    1 USD\n    Equity\n");
    let (ctx, _ledger) = process(arena, &content);
    let account = ctx.account(account_name).unwrap();
    (ctx, account)
}
