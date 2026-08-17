//! The query the report session runs with: the owned [`QueryOptions`] the user
//! states (on the command line, or in the `.` form) and the [`QueryState`] they
//! resolve to against a session's [`ReportContext`].
//!
//! The two exist separately because they have different lifetimes. A
//! [`Conversion`] names an interned commodity of one session's context, so it
//! dies with the arena a reload resets; [`QueryOptions`] is plain owned data
//! that crosses that boundary and is re-resolved against the session that
//! follows — the same split [`UiSnapshot`](super::app::UiSnapshot) makes for
//! the UI state.

use std::path::PathBuf;

use chrono::NaiveDate;
use okane_core::report::query::{
    AccountFilter, BalanceQuery, Conversion, ConversionStrategy, DateRange, QueryError,
};
use okane_core::report::{OwnedCommodity, ProcessOptions, ReportContext};

use super::register::RegisterQueryTemplate;

/// The report query as the user states it: the subset of the CLI's
/// `EvalOptions` that decides *what the numbers are*, as fully owned data.
///
/// The TUI lets every field be changed mid-session through the `.` form. All
/// but one are re-queried in place; [`Self::price_db`] feeds the book-keeping
/// rather than the query, so changing it means processing the source again —
/// see [`Self::needs_reprocess`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryOptions {
    /// Path to the price DB, as `--price-db`.
    pub price_db: Option<PathBuf>,
    /// Commodity every amount is converted to, as `-X` / `--exchange`.
    pub exchange: Option<String>,
    /// Convert at the date of each transaction (`--historical`) rather than at
    /// [`Self::today`]'s rate.
    pub historical: bool,
    /// The date "now" means, as `--today`. Not editable in the form: it is what
    /// a non-historical conversion is dated at, and the CLI's default (the
    /// local date) is the answer in a session someone is sitting in front of.
    pub today: NaiveDate,
    /// Start of the date range (inclusive), as `--start`.
    pub start: Option<NaiveDate>,
    /// End of the date range (exclusive), as `--end`.
    pub end: Option<NaiveDate>,
}

impl QueryOptions {
    pub fn to_process_options(&self) -> ProcessOptions {
        ProcessOptions {
            price_db_path: self.price_db.clone(),
        }
    }

    pub fn date_range(&self) -> DateRange {
        DateRange {
            start: self.start,
            end: self.end,
        }
    }

    fn conversion_strategy(&self) -> ConversionStrategy {
        if self.historical {
            ConversionStrategy::Historical
        } else {
            ConversionStrategy::UpToDate { today: self.today }
        }
    }

    /// Resolves these options against `ctx`, interning nothing: the target
    /// commodity must already be known to the session, which it is for any
    /// commodity the file (or its price DB) mentions. Fails with
    /// [`QueryError::CommodityNotFound`] otherwise — the answer to `-X` on a
    /// commodity that appears nowhere would be an empty report, not a report.
    pub(super) fn resolve<'ctx>(
        &self,
        ctx: &ReportContext<'ctx>,
    ) -> Result<QueryState<'ctx>, QueryError> {
        let conversion = match &self.exchange {
            None => None,
            Some(commodity) => {
                let target = ctx.commodity(commodity).ok_or_else(|| {
                    QueryError::CommodityNotFound(OwnedCommodity::from_string(commodity.clone()))
                })?;
                Some(Conversion {
                    strategy: self.conversion_strategy(),
                    target,
                })
            }
        };
        Ok(QueryState {
            options: self.clone(),
            template: RegisterQueryTemplate {
                conversion,
                date_range: self.date_range(),
            },
        })
    }

    /// Whether moving from `current` to these options requires the source to be
    /// processed again rather than merely re-queried. Only the price DB does:
    /// the rates are read while book-keeping, so a session already built has no
    /// way to acquire the new ones.
    pub(super) fn needs_reprocess(&self, current: &Self) -> bool {
        self.price_db != current.price_db
    }

    /// One-line summary for the status bar of the options that change what the
    /// numbers *mean* — the conversion and the date range. `None` for the plain
    /// report over the whole file, which needs no announcement.
    ///
    /// The price DB is deliberately left out: it is where the rates come from
    /// rather than a lens on the report, and it only shows up here as the `-X`
    /// it makes possible. The form is where to check it.
    pub(super) fn summary(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if let Some(commodity) = &self.exchange {
            parts.push(match self.historical {
                true => format!("-X {commodity} (historical)"),
                false => format!("-X {commodity}"),
            });
        }
        if self.start.is_some() || self.end.is_some() {
            parts.push(format!(
                "{}..{}",
                format_date(self.start),
                format_date(self.end)
            ));
        }
        (!parts.is_empty()).then(|| parts.join(" · "))
    }
}

/// A date as the form types it and the status bar prints it; an open end of a
/// range is left blank.
pub(super) fn format_date(date: Option<NaiveDate>) -> String {
    date.map(|d| d.format(DATE_FORMAT).to_string())
        .unwrap_or_default()
}

/// The one date format, shared by the form's parser and its placeholder — the
/// same ISO form `--start` / `--end` take on the command line.
pub(super) const DATE_FORMAT: &str = "%Y-%m-%d";

/// Parses a date the way the CLI flags do, reporting the expected shape.
pub(super) fn parse_date(text: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(text, DATE_FORMAT)
        .map_err(|_| format!("expected a date as YYYY-MM-DD, got `{text}`"))
}

/// [`QueryOptions`] resolved against one session's context: the options as
/// stated, plus the `'ctx` query parameters derived from them.
///
/// Held by the [`App`](super::app::App) so the two can never drift: every path
/// that changes the options builds a new state from them.
#[derive(Debug, Clone)]
pub struct QueryState<'ctx> {
    pub options: QueryOptions,
    /// Query parameters shared by every register lookup of the session.
    pub template: RegisterQueryTemplate<'ctx>,
}

impl<'ctx> QueryState<'ctx> {
    /// The state of a session that has no data to resolve against — the empty
    /// session a failed reload leaves behind. The options are kept as stated
    /// (the form opens on them, which is how a bad `--price-db` gets fixed),
    /// but nothing is converted, since there is no context to intern against.
    pub(super) fn unresolved(options: &QueryOptions) -> Self {
        Self {
            options: options.clone(),
            template: RegisterQueryTemplate {
                conversion: None,
                date_range: options.date_range(),
            },
        }
    }

    /// The balance query for the whole ledger under these options. The account
    /// filter is always [`AccountFilter::All`]: the balance screen shows every
    /// account and narrows by folding and searching rather than by re-querying.
    pub(super) fn balance_query(&self) -> BalanceQuery<'ctx> {
        BalanceQuery {
            account: AccountFilter::All,
            conversion: self.template.conversion,
            date_range: self.template.date_range,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use bumpalo::Bump;

    use super::super::testing::{options, process};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    const LEDGER: &str = "2024/01/01 Init\n    Assets:Bank    10 USD\n    Equity\n";

    #[test]
    fn resolve_without_exchange_has_no_conversion() {
        let arena = Bump::new();
        let (ctx, _ledger) = process(&arena, LEDGER);
        let state = options().resolve(&ctx).unwrap();
        assert!(state.template.conversion.is_none());
    }

    #[test]
    fn resolve_picks_the_strategy_from_historical() {
        let arena = Bump::new();
        let (ctx, _ledger) = process(&arena, LEDGER);

        let mut opts = options();
        opts.exchange = Some("USD".to_owned());
        let state = opts.resolve(&ctx).unwrap();
        assert_matches!(
            state.template.conversion,
            Some(Conversion {
                strategy: ConversionStrategy::UpToDate { today },
                ..
            }) if today == date(2024, 6, 1)
        );

        opts.historical = true;
        let state = opts.resolve(&ctx).unwrap();
        assert_matches!(
            state.template.conversion,
            Some(Conversion {
                strategy: ConversionStrategy::Historical,
                ..
            })
        );
    }

    /// A commodity the ledger never mentions cannot be interned, and a report
    /// converted into it would be empty rather than wrong — so it is refused.
    #[test]
    fn resolve_unknown_commodity_fails() {
        let arena = Bump::new();
        let (ctx, _ledger) = process(&arena, LEDGER);
        let mut opts = options();
        opts.exchange = Some("XYZ".to_owned());
        assert_matches!(
            opts.resolve(&ctx),
            Err(QueryError::CommodityNotFound(commodity)) if commodity.to_string() == "XYZ"
        );
    }

    #[test]
    fn only_the_price_db_needs_a_reprocess() {
        let current = options();
        let mut next = current.clone();
        next.exchange = Some("CHF".to_owned());
        next.historical = true;
        next.start = Some(date(2024, 1, 1));
        next.end = Some(date(2025, 1, 1));
        assert!(!next.needs_reprocess(&current));

        next.price_db = Some(PathBuf::from("prices.db"));
        assert!(next.needs_reprocess(&current));
    }

    #[test]
    fn summary_of_the_plain_report_is_empty() {
        let mut opts = options();
        assert_eq!(opts.summary(), None);
        // The price DB is a source of rates, not a lens: on its own it changes
        // nothing about what the status bar has to explain.
        opts.price_db = Some(PathBuf::from("prices.db"));
        assert_eq!(opts.summary(), None);
    }

    #[test]
    fn summary_names_the_conversion_and_the_range() {
        let mut opts = options();
        opts.exchange = Some("CHF".to_owned());
        assert_eq!(opts.summary().as_deref(), Some("-X CHF"));
        opts.historical = true;
        assert_eq!(opts.summary().as_deref(), Some("-X CHF (historical)"));

        opts.start = Some(date(2024, 1, 1));
        assert_eq!(
            opts.summary().as_deref(),
            Some("-X CHF (historical) · 2024-01-01..")
        );
        opts.end = Some(date(2025, 1, 1));
        assert_eq!(
            opts.summary().as_deref(),
            Some("-X CHF (historical) · 2024-01-01..2025-01-01")
        );

        let mut opts = options();
        opts.end = Some(date(2025, 1, 1));
        assert_eq!(opts.summary().as_deref(), Some("..2025-01-01"));
    }

    #[test]
    fn parse_date_round_trips_the_printed_form() {
        let d = date(2024, 12, 31);
        assert_eq!(parse_date(&format_date(Some(d))), Ok(d));
        assert_eq!(format_date(None), "");
    }

    #[test]
    fn parse_date_rejects_other_shapes() {
        // The ledger date style is not the flag's; be explicit about which one
        // the form wants rather than guessing at the intent.
        assert_matches!(parse_date("2024/12/31"), Err(msg) if msg.contains("YYYY-MM-DD"));
        assert_matches!(parse_date("nonsense"), Err(_));
    }
}
