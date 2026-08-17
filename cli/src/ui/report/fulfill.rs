//! Fulfillment of the [`Command`]s [`App::update`] asks for — the one impure
//! step of the loop, and the only place that touches the `Ledger` the pure
//! state machine cannot.
//!
//! Every command here can fail against real data (a register the current
//! conversion cannot express, options naming a commodity the file does not
//! have), and none of those failures is a reason to tear the session down:
//! they leave the shown report exactly as it was and say what happened in the
//! footer, which is why [`fulfill`] is infallible.

use lender::FallibleLender;
use okane_core::report::ReportContext;
use okane_core::report::query::{AccountFilter, Ledger, RegisterQuery, Sort};

use super::app::{App, Command};
use super::options::QueryOptions;
use super::register::{RegisterQueryTemplate, RegisterRow, RegisterScope};

/// What the event loop should do after a command has been fulfilled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Fulfilled {
    /// Keep the session running.
    Continue,
    /// Tear the session down and build a fresh one — the reload key, or an
    /// options change that only a reprocess can apply.
    Reload,
}

/// Carries out `cmd` against the live session.
pub(super) fn fulfill<'ctx>(
    cmd: Command<'ctx>,
    app: &mut App<'ctx>,
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> Fulfilled {
    match cmd {
        Command::Reload => Fulfilled::Reload,
        Command::LoadRegister { scope } => {
            match load_register(ledger, ctx, &app.query.template, scope) {
                Ok(rows) => app.show_register(scope, rows),
                Err(err) => app.error_toast = Some(register_error(scope, err.as_ref())),
            }
            Fulfilled::Continue
        }
        Command::ApplyOptions(options) => apply_options(options, app, ledger, ctx),
    }
}

/// Re-runs the report under `options`.
///
/// A different price DB is a different *book-keeping* of the source rather than
/// a different query of it, so it goes back to the session loop to be built
/// again; the options ride along on the app, which outlives this session's
/// data. Everything else is re-queried against the ledger already in memory —
/// no re-parse, and the file on disk is left alone, which is what keeps this
/// distinct from a reload.
fn apply_options<'ctx>(
    options: QueryOptions,
    app: &mut App<'ctx>,
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
) -> Fulfilled {
    if options.needs_reprocess(&app.query.options) {
        app.query.options = options;
        return Fulfilled::Reload;
    }
    // The whole view is rebuilt from the new query and the old UI state, the
    // same way a reload rebuilds it — including re-running an open register.
    let snapshot = app.snapshot();
    match super::build_app(ctx, ledger, &options, &app.source_display, Some(&snapshot)) {
        Ok(next) => *app = next,
        // Nothing was swapped in, so the report on screen is still the one its
        // options describe.
        Err(err) => {
            app.error_toast = Some(format!(
                "failed to apply the options: {}",
                super::error_summary(err.as_ref())
            ));
        }
    }
    Fulfilled::Continue
}

/// Footer notice for a register that could not be loaded: which account, and
/// why, on one line.
pub(super) fn register_error(
    scope: RegisterScope<'_>,
    err: &(dyn std::error::Error + 'static),
) -> String {
    format!(
        "failed to load register for {}: {}",
        scope.display_name(),
        super::error_summary(err)
    )
}

/// Collects the register rows for `scope` into owned [`RegisterRow`]s so they
/// can be displayed without keeping the `FallibleLender` alive.
pub fn load_register<'ctx>(
    ledger: &mut Ledger<'ctx>,
    ctx: &ReportContext<'ctx>,
    template: &RegisterQueryTemplate<'ctx>,
    scope: RegisterScope<'ctx>,
) -> anyhow::Result<Vec<RegisterRow<'ctx>>> {
    let account = match scope {
        RegisterScope::Single(account) => AccountFilter::single(account),
        RegisterScope::Subtree(aggregate) => AccountFilter::descendants_of(ctx, aggregate),
        RegisterScope::All => AccountFilter::All,
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

    use assert_matches::assert_matches;
    use bumpalo::Bump;
    use chrono::NaiveDate;
    use indoc::indoc;

    use super::super::app::Screen;
    use super::super::testing::{options, process};

    /// Two years of postings, so a date range has something to cut, in two
    /// commodities — priced against each other — so a conversion has something
    /// to do. `Assets:Bank` ends up holding 110 USD and 5 EUR.
    const LEDGER: &str = indoc! {"
        2024/01/01 Init
            Assets:Bank    10 USD
            Equity

        2024/02/01 Exchange
            Assets:Bank    5 EUR @ 2 USD
            Equity

        2025/01/01 Later
            Assets:Bank    100 USD
            Equity
    "};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// Builds the app the way a session does, from `options`.
    fn app_with<'ctx>(
        ctx: &ReportContext<'ctx>,
        ledger: &mut Ledger<'ctx>,
        options: &QueryOptions,
    ) -> App<'ctx> {
        super::super::build_app(ctx, ledger, options, "test.ledger", None).unwrap()
    }

    /// The balance rows as `name: amount` text, which is what an options change
    /// is supposed to move.
    fn rows_of(app: &App<'_>, ctx: &ReportContext<'_>) -> Vec<String> {
        app.balance
            .rows
            .iter()
            .map(|row| format!("{}: {}", row.full_name(), row.amount.as_inline_display(ctx)))
            .collect()
    }

    #[test]
    fn applying_a_date_range_requeries_in_place() {
        let arena = Bump::new();
        let (ctx, mut ledger) = process(&arena, LEDGER);
        let mut app = app_with(&ctx, &mut ledger, &options());
        assert!(
            rows_of(&app, &ctx)
                .iter()
                .any(|row| row.contains("110 USD")),
            "{:?}",
            rows_of(&app, &ctx)
        );

        let mut next = options();
        next.end = Some(date(2025, 1, 1));
        assert_eq!(
            fulfill(
                Command::ApplyOptions(next.clone()),
                &mut app,
                &mut ledger,
                &ctx
            ),
            Fulfilled::Continue
        );

        assert_eq!(app.error_toast, None);
        assert_eq!(app.query.options, next);
        // The 2025 posting is outside the range now.
        let rows = rows_of(&app, &ctx);
        assert!(
            rows.iter().any(|row| row.contains("10 USD")),
            "the 2024 postings should still be there: {rows:?}"
        );
        assert!(
            rows.iter().all(|row| !row.contains("110 USD")),
            "the 2025 posting should be gone: {rows:?}"
        );
    }

    #[test]
    fn applying_an_exchange_converts_the_balance() {
        let arena = Bump::new();
        let (ctx, mut ledger) = process(&arena, LEDGER);
        let mut app = app_with(&ctx, &mut ledger, &options());

        let mut next = options();
        next.exchange = Some("USD".to_owned());
        next.historical = true;
        fulfill(Command::ApplyOptions(next), &mut app, &mut ledger, &ctx);

        assert_eq!(app.error_toast, None);
        let rows = rows_of(&app, &ctx);
        assert!(rows.iter().all(|row| !row.contains("EUR")), "{rows:?}");
        // 110 USD plus 5 EUR at the 2 USD rate it was bought at.
        assert!(rows.iter().any(|row| row.contains("120 USD")), "{rows:?}");
    }

    /// The selection, the tree state and the open register survive an options
    /// change the same way they survive a reload.
    #[test]
    fn applying_options_keeps_the_ui_state() {
        let arena = Bump::new();
        let (ctx, mut ledger) = process(&arena, LEDGER);
        let mut app = app_with(&ctx, &mut ledger, &options());
        let scope = app
            .balance
            .rows
            .iter()
            .find(|row| row.full_name() == "Assets:Bank")
            .map(|row| row.scope)
            .expect("the fixture has Assets:Bank");
        let rows = load_register(&mut ledger, &ctx, &app.query.template, scope).unwrap();
        app.show_register(scope, rows);

        let mut next = options();
        next.end = Some(date(2025, 1, 1));
        fulfill(Command::ApplyOptions(next), &mut app, &mut ledger, &ctx);

        assert_eq!(app.error_toast, None);
        assert_matches!(&app.screen, Screen::Register(view) => {
            assert_eq!(view.title(), "Assets:Bank");
            // Re-queried under the new range: the 2025 entry is gone.
            assert_eq!(view.rows.len(), 2);
        });
    }

    /// A commodity the file never mentions cannot be converted into. The
    /// refusal leaves the report — and the options that describe it — alone.
    #[test]
    fn applying_an_unknown_commodity_keeps_the_old_report() {
        let arena = Bump::new();
        let (ctx, mut ledger) = process(&arena, LEDGER);
        let before = options();
        let mut app = app_with(&ctx, &mut ledger, &before);
        let rows = rows_of(&app, &ctx);

        let mut next = options();
        next.exchange = Some("XYZ".to_owned());
        assert_eq!(
            fulfill(Command::ApplyOptions(next), &mut app, &mut ledger, &ctx),
            Fulfilled::Continue
        );

        let toast = app.error_toast.as_deref().expect("a footer notice");
        assert!(toast.contains("XYZ"), "{toast}");
        assert_eq!(app.query.options, before);
        assert_eq!(rows_of(&app, &ctx), rows);
    }

    /// The price DB is read while book-keeping, so a new one goes back to the
    /// session loop — with the options attached, since the app is what crosses
    /// that boundary.
    #[test]
    fn applying_a_price_db_asks_for_a_rebuild() {
        let arena = Bump::new();
        let (ctx, mut ledger) = process(&arena, LEDGER);
        let mut app = app_with(&ctx, &mut ledger, &options());

        let mut next = options();
        next.price_db = Some("prices.db".into());
        assert_eq!(
            fulfill(
                Command::ApplyOptions(next.clone()),
                &mut app,
                &mut ledger,
                &ctx
            ),
            Fulfilled::Reload
        );
        assert_eq!(app.query.options, next);
    }

    /// A register the conversion cannot express is a notice, not the end of the
    /// session: an up-to-date `-X` is unsupported there (see okane#313).
    #[test]
    fn a_register_the_conversion_cannot_express_only_warns() {
        let arena = Bump::new();
        let (ctx, mut ledger) = process(&arena, LEDGER);
        let mut opts = options();
        opts.exchange = Some("USD".to_owned()); // up-to-date, not historical
        let mut app = app_with(&ctx, &mut ledger, &opts);
        let scope = app
            .balance
            .rows
            .iter()
            .find(|row| row.full_name() == "Assets:Bank")
            .map(|row| row.scope)
            .expect("the fixture has Assets:Bank");

        assert_eq!(
            fulfill(Command::LoadRegister { scope }, &mut app, &mut ledger, &ctx),
            Fulfilled::Continue
        );
        assert_matches!(&app.screen, Screen::Balance);
        let toast = app.error_toast.as_deref().expect("a footer notice");
        assert!(toast.contains("Assets:Bank"), "{toast}");
    }
}
