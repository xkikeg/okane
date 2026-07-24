//! Report TUI session — balance and register viewer.
//!
//! Architecture (Elm-style; see
//! <https://ratatui.rs/concepts/application-patterns/the-elm-architecture/>):
//! - [`app::App`] holds all state and exposes [`app::App::update`] for transitions.
//! - [`event`] translates `KeyEvent`s to messages and runs the loop.
//! - [`render`] is a pure view over `App`.
//!
//! Reload (`r` / `F5`) is a *session* boundary: all report data — the
//! [`ReportContext`] intern stores (accounts, commodities, aliases) and
//! everything referencing the arena — is torn down, the arena is reset, and
//! a fresh session is built from scratch, so stale interned entries never
//! survive a reload. Only an owned [`app::UiSnapshot`] (selection, search,
//! screen) crosses the boundary. See [`run_ui`] for the loop.

mod app;
mod event;
mod render;

pub use app::{App, RegisterQueryTemplate};

use bumpalo::Bump;
use okane_core::load;
use okane_core::report::query::{
    AccountFilter, BalanceQuery, Conversion, ConversionStrategy, DateRange, Ledger, QueryError,
};
use okane_core::report::{self, OwnedCommodity, ProcessOptions, ReportContext};
use ratatui::DefaultTerminal;

use app::{ErrorPopup, Overlay, UiSnapshot};

/// Everything needed to build (and rebuild) a session: the loader re-reads
/// the source from disk on every `load` call, and the options carry the CLI
/// configuration. Fully owned — no arena references — so it survives the
/// arena resets between sessions.
pub struct SessionConfig<F: load::FileSystem> {
    /// Loader shared by the first build and every reload. Errors from the
    /// first build print to the normal terminal (before it's set up);
    /// errors from a reload are shown in the TUI's error modal, which
    /// renders the (styled) ANSI output via `ansi-to-tui`.
    loader: load::Loader<F>,
    process_options: ProcessOptions,
    /// `--exchange` conversion as owned data, re-resolved against each
    /// session's fresh [`ReportContext`].
    conversion: Option<ConversionSpec>,
    date_range: DateRange,
}

impl<F: load::FileSystem> SessionConfig<F> {
    /// Wraps a loader for the source plus the query configuration.
    pub fn new(
        loader: load::Loader<F>,
        process_options: ProcessOptions,
        conversion: Option<ConversionSpec>,
        date_range: DateRange,
    ) -> Self {
        Self {
            loader,
            process_options,
            conversion,
            date_range,
        }
    }
}

/// Owned form of [`Conversion`]: unlike the latter, it holds no arena
/// reference, so it can be resolved against every session's context anew.
pub struct ConversionSpec {
    pub commodity: String,
    pub strategy: ConversionStrategy,
}

impl ConversionSpec {
    fn resolve<'ctx>(&self, ctx: &ReportContext<'ctx>) -> Result<Conversion<'ctx>, QueryError> {
        let target = ctx.commodity(&self.commodity).ok_or_else(|| {
            QueryError::CommodityNotFound(OwnedCommodity::from_string(self.commodity.clone()))
        })?;
        Ok(Conversion {
            strategy: self.strategy,
            target,
        })
    }
}

/// Runs the TUI session loop.
///
/// Each iteration owns one session: a fresh [`ReportContext`] over the
/// (reset) arena and the data built from it. A reload ends the iteration,
/// so every `'ctx` borrow dies with it and `arena.reset()` is safe; UI
/// state crosses over as an owned [`UiSnapshot`].
///
/// The terminal (raw mode, alternate screen; panic hook installed by
/// `ratatui::init`) is entered lazily once the first session builds, so
/// startup errors print to the normal terminal. It is restored on exit
/// (normal or error).
pub fn run_ui<F: load::FileSystem>(
    arena: &mut Bump,
    source_display: String,
    config: &SessionConfig<F>,
) -> anyhow::Result<()> {
    let mut terminal = None;
    let result = session_loop(arena, &source_display, config, &mut terminal);
    if terminal.is_some() {
        ratatui::restore();
    }
    result
}

fn session_loop<F: load::FileSystem>(
    arena: &mut Bump,
    source_display: &str,
    config: &SessionConfig<F>,
    terminal: &mut Option<DefaultTerminal>,
) -> anyhow::Result<()> {
    let mut snapshot: Option<UiSnapshot> = None;
    let mut first = true;
    loop {
        if !first {
            arena.reset();
        }
        let mut ctx = ReportContext::new(arena);
        let built = build_session(&mut ctx, config, source_display, snapshot.as_ref());
        let (mut ledger, mut app, has_data) = match built {
            Ok(SessionData { ledger, app }) => (ledger, app, true),
            // A startup failure aborts before the terminal is set up.
            Err(err) if first => return Err(err),
            // A failed reload shows an empty session with the error in a modal;
            // `r` retries from there. The last snapshot is kept so a later
            // successful reload still restores the pre-error UI state.
            Err(err) => {
                let template = RegisterQueryTemplate {
                    conversion: None,
                    date_range: config.date_range,
                };
                let mut app = App::new(source_display.to_owned(), Vec::new(), template);
                app.overlay = Some(error_overlay(source_display, err.as_ref()));
                (Ledger::empty(&ctx), app, false)
            }
        };
        first = false;
        let terminal = terminal.get_or_insert_with(ratatui::init);
        match event::run(terminal, &mut app, &mut ledger, &ctx)? {
            event::RunOutcome::Quit => return Ok(()),
            event::RunOutcome::Reload => {
                if has_data {
                    snapshot = Some(app.snapshot());
                }
            }
        }
    }
}

/// One session's worth of arena-backed data.
#[derive(Debug)]
struct SessionData<'ctx> {
    ledger: Ledger<'ctx>,
    app: App<'ctx>,
}

/// Loads and processes the source into `ctx` and builds the session data,
/// restoring the UI state from `snapshot` when given.
fn build_session<'ctx, F: load::FileSystem>(
    ctx: &mut ReportContext<'ctx>,
    config: &SessionConfig<F>,
    source_display: &str,
    snapshot: Option<&UiSnapshot>,
) -> anyhow::Result<SessionData<'ctx>> {
    let mut ledger = report::process(ctx, &config.loader, &config.process_options)?;
    let conversion = config
        .conversion
        .as_ref()
        .map(|spec| spec.resolve(ctx))
        .transpose()?;
    let query = BalanceQuery {
        account: AccountFilter::All,
        conversion,
        date_range: config.date_range,
    };
    let balance = ledger.balance(ctx, &query)?.into_owned();
    let tree = report::BalanceTree::create(ctx, balance)?.into_nodes();
    let template = RegisterQueryTemplate {
        conversion,
        date_range: config.date_range,
    };
    let mut app = App::with_tree(source_display.to_owned(), tree, template);
    if let Some(snapshot) = snapshot
        && let Some((drill, title, index)) = app.restore(snapshot)
    {
        // The snapshot had the register screen open; re-query its rows
        // against the fresh data.
        match event::load_register(&mut ledger, ctx, &app.register_template, drill) {
            Ok(rows) => app.show_register_at(drill, title, rows, index),
            Err(err) => {
                app.error_toast = Some(format!(
                    "failed to load register for {}: {}",
                    title,
                    error_summary(err.as_ref())
                ));
            }
        }
    }
    Ok(SessionData { ledger, app })
}

/// One-line summary of an error chain, suitable for the TUI footer. Each
/// cause can render multi-line (annotate-snippets), so only the first line
/// of each is kept.
fn error_summary(err: &(dyn std::error::Error + 'static)) -> String {
    let mut summary = first_line(&err.to_string());
    let mut source = err.source();
    while let Some(err) = source {
        summary.push_str(": ");
        summary.push_str(&first_line(&err.to_string()));
        source = err.source();
    }
    summary
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or_default().to_owned()
}

/// Full rendering of an error chain, for the error modal.
///
/// Unlike [`error_summary`], every cause's complete `Display` is kept: for a
/// parse or book-keeping failure the useful part — the annotate-snippets
/// source excerpt — lives entirely past the first line, and
/// `ReportError::BookKeep` has no `source()` at all, so its snippet *is* the
/// whole message. Returned pre-split into display lines for [`ErrorPopup`].
fn error_detail(err: &(dyn std::error::Error + 'static)) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = Some(err);
    let mut depth = 0usize;
    while let Some(err) = current {
        if depth > 0 {
            lines.push(String::new());
            lines.push(format!("caused by ({depth}):"));
        }
        lines.extend(err.to_string().lines().map(str::to_owned));
        current = err.source();
        depth += 1;
    }
    lines
}

/// Builds the modal shown when loading `source_display` failed.
///
/// The title carries only the file name: a long absolute path would fill the
/// whole title bar and truncate. The full path stays visible in the window
/// title and in the error body's own location line.
fn error_overlay(source_display: &str, err: &(dyn std::error::Error + 'static)) -> Overlay {
    let name = std::path::Path::new(source_display)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(source_display);
    Overlay::Error(ErrorPopup::new(
        format!(" failed to load {name} "),
        error_detail(err),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use assert_matches::assert_matches;
    use okane_core::load::FakeFileSystem;

    use app::{Drill, Screen};

    const V1: &str = "2024/01/01 Init\n    Assets:Bank    10 USD\n    Assets:Cash    5 USD\n    Expenses:Food    3 USD\n    Equity\n";
    // V1 plus a new account sorting before all others.
    const V2: &str = "2024/01/01 Init\n    Assets:Aaa    1 USD\n    Assets:Bank    10 USD\n    Assets:Cash    5 USD\n    Expenses:Food    3 USD\n    Equity\n";
    // V1 without Expenses:Food.
    const V3: &str =
        "2024/01/01 Init\n    Assets:Bank    10 USD\n    Assets:Cash    5 USD\n    Equity\n";
    const BROKEN: &str = "@@ this is not a valid ledger file @@\n";
    // Parses fine, fails book-keeping — `ReportError::BookKeep`, whose whole
    // payload is its `Display` (no `source()` behind it).
    const UNBALANCED: &str =
        "2024/01/01 Groceries\n    Expenses:Food    30 CHF\n    Assets:Bank    -25 CHF\n";

    fn fake_loader(content: &str) -> load::Loader<FakeFileSystem> {
        let mut map = HashMap::new();
        map.insert(PathBuf::from("test.ledger"), content.as_bytes().to_vec());
        load::Loader::new(
            PathBuf::from("test.ledger"),
            load::FakeFileSystem::from(map),
        )
    }

    fn session_config(content: &str) -> SessionConfig<FakeFileSystem> {
        SessionConfig::new(
            fake_loader(content),
            ProcessOptions::default(),
            None,
            DateRange::default(),
        )
    }

    /// Builds one session from `content` the way the session loop does.
    fn build<'ctx>(
        ctx: &mut ReportContext<'ctx>,
        content: &str,
        snapshot: Option<&UiSnapshot>,
    ) -> anyhow::Result<SessionData<'ctx>> {
        let config = session_config(content);
        build_session(ctx, &config, "test", snapshot)
    }

    fn account_names(app: &App<'_>) -> Vec<String> {
        app.balance_rows
            .iter()
            .map(|r| r.full_name.to_owned())
            .collect()
    }

    #[test]
    fn rebuild_swaps_rows_and_follows_selection() {
        let mut arena = Bump::new();
        let snapshot = {
            let mut ctx = ReportContext::new(&arena);
            let SessionData { mut app, .. } = build(&mut ctx, V1, None).unwrap();
            assert_eq!(
                account_names(&app),
                ["Assets:Bank", "Assets:Cash", "Equity", "Expenses:Food"]
            );
            app.balance_nav.select(3); // Expenses:Food
            app.snapshot()
        };
        arena.reset();

        let mut ctx = ReportContext::new(&arena);
        let SessionData { app, .. } = build(&mut ctx, V2, Some(&snapshot)).unwrap();

        assert_eq!(app.error_toast, None);
        assert_eq!(
            account_names(&app),
            [
                "Assets:Aaa",
                "Assets:Bank",
                "Assets:Cash",
                "Equity",
                "Expenses:Food"
            ]
        );
        // The selection followed Expenses:Food to its new index.
        assert_eq!(app.balance_nav.table_state.selected(), Some(4));
    }

    /// The reason reload rebuilds from scratch: interned entries of the
    /// previous session (accounts, and likewise aliases) must not leak
    /// into the fresh context.
    #[test]
    fn rebuild_drops_stale_interned_accounts() {
        let mut arena = Bump::new();
        {
            let mut ctx = ReportContext::new(&arena);
            build(&mut ctx, V1, None).unwrap();
            assert!(ctx.account("Expenses:Food").is_some());
        }
        arena.reset();

        let mut ctx = ReportContext::new(&arena);
        build(&mut ctx, V3, None).unwrap();
        assert_eq!(ctx.account("Expenses:Food"), None);
    }

    #[test]
    fn rebuild_reopens_register_screen() {
        let mut arena = Bump::new();
        let snapshot = {
            let mut ctx = ReportContext::new(&arena);
            let SessionData {
                mut app,
                mut ledger,
            } = build(&mut ctx, V1, None).unwrap();
            let account = ctx.account("Assets:Bank").unwrap();
            let drill = Drill::Single(account);
            let rows =
                event::load_register(&mut ledger, &ctx, &app.register_template, drill).unwrap();
            app.show_register(drill, account.as_str().to_owned(), rows);
            app.snapshot()
        };
        arena.reset();

        let mut ctx = ReportContext::new(&arena);
        let SessionData { app, .. } = build(&mut ctx, V2, Some(&snapshot)).unwrap();

        assert_eq!(app.error_toast, None);
        let Screen::Register(view) = &app.screen else {
            panic!("expected register screen");
        };
        assert_eq!(view.title, "Assets:Bank");
        assert_eq!(view.rows.len(), 1);
        assert_eq!(view.nav.table_state.selected(), Some(0));
    }

    #[test]
    fn rebuild_register_account_vanished_returns_to_balance() {
        let mut arena = Bump::new();
        let snapshot = {
            let mut ctx = ReportContext::new(&arena);
            let SessionData {
                mut app,
                mut ledger,
            } = build(&mut ctx, V1, None).unwrap();
            let account = ctx.account("Expenses:Food").unwrap();
            let drill = Drill::Single(account);
            let rows =
                event::load_register(&mut ledger, &ctx, &app.register_template, drill).unwrap();
            app.show_register(drill, account.as_str().to_owned(), rows);
            app.snapshot()
        };
        arena.reset();

        let mut ctx = ReportContext::new(&arena);
        let SessionData { app, .. } = build(&mut ctx, V3, Some(&snapshot)).unwrap();

        assert_matches!(&app.screen, Screen::Balance);
        assert_matches!(&app.error_toast, Some(_));
        assert_eq!(
            account_names(&app),
            ["Assets:Bank", "Assets:Cash", "Equity"]
        );
    }

    #[test]
    fn build_session_broken_input_fails() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let err = build(&mut ctx, BROKEN, None).unwrap_err();
        // The footer summary is a single line even though the parse error
        // renders as a multi-line snippet.
        assert!(!error_summary(err.as_ref()).contains('\n'));
    }

    #[test]
    fn error_detail_keeps_the_whole_snippet() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let err = build(&mut ctx, BROKEN, None).unwrap_err();
        let lines = error_detail(err.as_ref());
        // Far more than the one line per cause `error_summary` keeps.
        assert!(lines.len() > 3, "too few lines: {lines:#?}");
        let text = lines.join("\n");
        // The offending source line is what makes the modal worth reading.
        assert!(text.contains("this is not a valid ledger file"), "{text}");
        assert!(text.contains("caused by"), "{text}");
    }

    /// The case that motivated the modal: `error_summary` reduces this to
    /// `"failed to balance the transaction"`, throwing the snippet away.
    #[test]
    fn error_detail_keeps_book_keeping_snippet() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let err = build(&mut ctx, UNBALANCED, None).unwrap_err();
        assert!(!error_summary(err.as_ref()).contains('\n'));

        let text = error_detail(err.as_ref()).join("\n");
        assert!(text.contains("unbalanced postings"), "{text}");
        // The source excerpt and its caret row are the point of the modal.
        assert!(text.contains("Assets:Bank    -25 CHF"), "{text}");
        assert!(text.contains('^'), "{text}");
    }

    /// Stands in for `ReportError::BookKeep`, whose whole payload is a
    /// multi-line `Display` with no `source()` behind it.
    #[derive(Debug)]
    struct Flat;

    impl std::fmt::Display for Flat {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "something broke\n  here:\n  ^^^^")
        }
    }

    impl std::error::Error for Flat {}

    #[test]
    fn error_detail_of_sourceless_error_has_no_caused_by() {
        assert_eq!(
            error_detail(&Flat),
            ["something broke", "  here:", "  ^^^^"]
        );
    }

    #[test]
    fn error_overlay_titles_the_source() {
        assert_matches!(
            error_overlay("main.ledger", &Flat),
            Overlay::Error(popup) => {
                assert!(popup.title.contains("failed to load main.ledger"), "{}", popup.title);
                assert_eq!(popup.lines.len(), 3);
                assert_eq!(popup.scroll, 0);
            }
        );
    }

    /// A long path would otherwise truncate the title to uselessness.
    #[test]
    fn error_overlay_title_uses_the_file_name_only() {
        assert_matches!(
            error_overlay("/some/very/long/path/to/main.ledger", &Flat),
            Overlay::Error(popup) => {
                assert_eq!(popup.title, " failed to load main.ledger ");
            }
        );
    }
}
