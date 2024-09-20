//! Provides [Tracking] decoration, which attaches [Tracked] span information
//! into syntax types so that report can raise error with the right position.

use std::fmt::Debug;
use std::ops::Range;

use super::decoration::{AsUndecorated, Decoration};

/// Tracking provides [Tracked] decoration for syntax types.
pub struct Tracking;

impl Decoration for Tracking {
    type Decorated<T> = Tracked<T>
    where
        T: AsUndecorated<T> + Debug + PartialEq + Eq;

    fn decorate_parser<PIn, I, O, E>(parser: PIn) -> impl winnow::Parser<I, Self::Decorated<O>, E>
    where
        I: winnow::stream::Stream + winnow::stream::Location,
        O: AsUndecorated<O> + Debug + PartialEq + Eq,
        PIn: winnow::Parser<I, O, E>,
    {
        use winnow::Parser;
        parser
            .with_span()
            .map(|(value, span)| Tracked::new(value, TrackedSpan(span)))
    }
}

pub type LedgerEntry<'i> = super::LedgerEntry<'i, Tracking>;
pub type Transaction<'i> = super::Transaction<'i, Tracking>;
pub type Posting<'i> = super::Posting<'i, Tracking>;
pub type PostingAmount<'i> = super::PostingAmount<'i, Tracking>;
pub type Lot<'i> = super::Lot<'i, Tracking>;

/// Span of the tracked position.
/// Only useful within [ParsedContext][crate::parse::ParsedContext].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TrackedSpan(Range<usize>);

impl TrackedSpan {
    /// Returns the implementation of the span.
    /// Note this is relative to the original string,
    /// not the local index in the string.
    pub(crate) fn as_range(&self) -> Range<usize> {
        self.0.clone()
    }

    /// Creates an instance, only for unit test.
    #[cfg(test)]
    pub fn new(span: Range<usize>) -> TrackedSpan {
        TrackedSpan(span)
    }
}

/// Tracked provides the struct with maybe position tracking.
#[derive(Debug, PartialEq, Eq)]
pub struct Tracked<T> {
    value: T,
    span: TrackedSpan,
}

impl<T> AsUndecorated<T> for Tracked<T> {
    fn as_undecorated(&self) -> &T {
        &self.value
    }
}

impl<T> Tracked<T> {
    /// Returns span for the tracked position.
    pub fn span(&self) -> TrackedSpan {
        self.span.clone()
    }

    /// Wraps another value with the same span.
    pub fn new(value: T, span: TrackedSpan) -> Self {
        Self { value, span }
    }
}
