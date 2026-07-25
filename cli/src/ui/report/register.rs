//! Register drill-down screen: its rows, the scope it is filtered to, and the
//! [`Screen`] focus enum that selects between the balance and register views.

use std::cmp::max;

use chrono::NaiveDate;
use okane_core::report::query::{Conversion, DateRange};
use okane_core::report::{Account, AccountAggregate, Amount};

use crate::ui::table::TableNav;

use super::balance::amount_line_count;

/// What a register drill-in from the balance screen should match.
#[derive(Debug, Clone, Copy)]
pub enum RegisterScope<'ctx> {
    /// Exactly one account (flat-view drill-in).
    Single(Account<'ctx>),
    /// An account and all of its descendants (tree-view drill-in).
    Subtree(AccountAggregate<'ctx>),
}

impl<'ctx> RegisterScope<'ctx> {
    /// The account/prefix name shown in the register title.
    pub fn display_name(&self) -> &'ctx str {
        match self {
            RegisterScope::Single(account) => account.as_str(),
            RegisterScope::Subtree(aggregate) => aggregate.as_str(),
        }
    }

    /// Owned form that survives a reload (see [`OwnedRegisterScope`]).
    pub(super) fn as_owned_scope(self) -> OwnedRegisterScope {
        match self {
            RegisterScope::Single(account) => {
                OwnedRegisterScope::Single(account.as_str().to_owned())
            }
            RegisterScope::Subtree(aggregate) => {
                OwnedRegisterScope::Subtree(aggregate.as_str().to_owned())
            }
        }
    }
}

/// Owned mirror of [`RegisterScope`] that survives a reload: the arena reset
/// invalidates the `'ctx` references, so the scope is kept by account name and
/// rebuilt with [`super::balance::BalanceView::resolve_scope`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedRegisterScope {
    /// A single account, by full name.
    Single(String),
    /// An account and all its descendants, by full name.
    Subtree(String),
}

impl OwnedRegisterScope {
    /// The account/prefix name the register was filtered to.
    pub(super) fn name(&self) -> &str {
        match self {
            OwnedRegisterScope::Single(name) | OwnedRegisterScope::Subtree(name) => name,
        }
    }
}

/// One row of the register table.
///
/// The account is implied by the active [`RegisterView`] (exact-match
/// filter), so it is not duplicated per row.
#[derive(Debug, Clone)]
pub struct RegisterRow<'ctx> {
    pub date: NaiveDate,
    pub payee: String,
    pub amount: Amount<'ctx>,
    pub total: Amount<'ctx>,
}

impl RegisterRow<'_> {
    /// Number of rendered lines this row occupies (>= 1).
    pub fn line_count(&self) -> u16 {
        max(
            amount_line_count(&self.amount),
            amount_line_count(&self.total),
        )
    }
}

/// Query parameters reused for every register lookup during the session
/// (built once from the CLI's `EvalOptions`).
#[derive(Debug, Clone, Copy)]
pub struct RegisterQueryTemplate<'ctx> {
    pub conversion: Option<Conversion<'ctx>>,
    pub date_range: DateRange,
}

/// State for the register drill-down screen.
#[derive(Debug)]
pub struct RegisterView<'ctx> {
    /// What this register is filtered to (single account or subtree). Also the
    /// source of the title-bar label (see [`Self::title`]).
    pub scope: RegisterScope<'ctx>,
    pub rows: Vec<RegisterRow<'ctx>>,
    pub nav: TableNav,
    /// Cached `(amount, total)` column widths. The amounts are fixed for the
    /// life of the view, so the renderer computes these once (scanning all
    /// rows) on first draw and reuses them, keeping per-frame work
    /// proportional to the viewport rather than the row count.
    pub col_widths: Option<(u16, u16)>,
}

impl<'ctx> RegisterView<'ctx> {
    pub fn new(scope: RegisterScope<'ctx>, rows: Vec<RegisterRow<'ctx>>) -> Self {
        let mut nav = TableNav::new(rows.len());
        // Most recent entry is the most useful starting point.
        nav.select_last();
        Self {
            scope,
            rows,
            nav,
            col_widths: None,
        }
    }

    /// The account/prefix name shown in the title bar (always the scope's name).
    pub fn title(&self) -> &'ctx str {
        self.scope.display_name()
    }
}

/// Top-level screen the user is currently looking at.
#[derive(Debug)]
pub enum Screen<'ctx> {
    Balance,
    Register(RegisterView<'ctx>),
}

/// Snapshot of register view that survives a reload, as part of
/// [`super::app::UiSnapshot`].
#[derive(Debug, Clone)]
pub struct RegisterSnapshot {
    /// What the register was filtered to (single account or subtree), owned so
    /// it survives the arena reset.
    pub(super) scope: OwnedRegisterScope,
    /// Selected cursor index.
    pub(super) cursor: usize,
}

impl RegisterSnapshot {
    /// Captures a register view's scope (owned) and cursor.
    pub(super) fn capture(view: &RegisterView<'_>) -> Self {
        Self {
            scope: view.scope.as_owned_scope(),
            cursor: view.nav.table_state.selected().unwrap_or(0),
        }
    }

    pub(super) fn scope(&self) -> &OwnedRegisterScope {
        &self.scope
    }

    pub(super) fn cursor(&self) -> usize {
        self.cursor
    }
}
