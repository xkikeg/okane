use std::fmt::Debug;
use std::ops::Range;

use super::decoration::{AsUndecorated, Decoration};

/// Tracking provides [Tracked] decoration for syntax types.
#[derive(Debug, PartialEq, Eq)]
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
            .map(|(value, position)| Tracked { position, value })
    }
}

pub type LedgerEntry<'i> = super::LedgerEntry<'i, Tracking>;
pub type Transaction<'i> = super::Transaction<'i, Tracking>;
pub type Posting<'i> = super::Posting<'i, Tracking>;
pub type PostingAmount<'i> = super::PostingAmount<'i, Tracking>;
pub type Lot<'i> = super::Lot<'i, Tracking>;

/// Tracked provides the struct with maybe position tracking.
#[derive(Debug, PartialEq, Eq)]
pub struct Tracked<T> {
    position: Range<usize>,
    value: T,
}

impl<T> AsUndecorated<T> for Tracked<T> {
    fn as_undecorated(&self) -> &T {
        &self.value
    }
}
